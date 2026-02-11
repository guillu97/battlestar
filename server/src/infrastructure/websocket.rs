use battlestar_shared::{ClientInput, ServerMessage};
use crate::app::AppState;
use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::{atomic::Ordering, Arc};
use std::time::{Duration, Instant};

/// WebSocket upgrade handler
///
/// Called when a client connects to /ws endpoint.
/// Upgrades HTTP connection to WebSocket and spawns handler.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle a single WebSocket connection
///
/// Lifecycle:
/// 1. Assign unique player ID (lock-free atomic)
/// 2. Send Welcome message with assigned ID
/// 3. Register player in game
/// 4. Run event loop (receive inputs, send game state)
/// 5. Cleanup on disconnect
///
/// Uses new architecture with single-lock pattern for game state access.
async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Assign unique player ID (lock-free)
    let player_id = state.next_player_id.fetch_add(1, Ordering::SeqCst);

    // Send Welcome message
    let welcome = ServerMessage::Welcome {
        assigned_id: player_id,
    };
    if let Ok(welcome_json) = serde_json::to_string(&welcome) {
        if socket
            .send(Message::Text(welcome_json.into()))
            .await
            .is_err()
        {
            return;
        }
    }

    // Register player in game (single lock)
    {
        let mut snapshot = state.game.lock().await;
        snapshot.connected_players.insert(player_id);
    }

    // Subscribe to game state broadcasts
    let mut rx = state.broadcaster.subscribe();

    // Event loop
    loop {
        tokio::select! {
            // Receive input from client
            maybe_msg = socket.recv() => {
                let Some(Ok(msg)) = maybe_msg else {
                    break; // Connection closed or error
                };

                if let Message::Text(text) = msg {
                    if let Ok(mut input) = serde_json::from_str::<ClientInput>(&text) {
                        // Override client's player_id with server-assigned ID (anti-cheat)
                        input.player_id = player_id;

                        // SERVER-SIDE RATE LIMITING (anti-cheat)
                        // Minimum 15ms between inputs (~66 inputs/sec max)
                        // Allows 60Hz client input with some tolerance
                        const MIN_INPUT_INTERVAL: Duration = Duration::from_millis(15);

                        let now = Instant::now();

                        // Single lock for rate limiting + input storage
                        let should_accept = {
                            let mut snapshot = state.game.lock().await;

                            // Check rate limit
                            if let Some(last_time) = snapshot.last_input_time.get(&player_id) {
                                let elapsed = now.duration_since(*last_time);
                                if elapsed < MIN_INPUT_INTERVAL {
                                    // Rate limit exceeded - reject input
                                    false
                                } else {
                                    // Update last input time and store input
                                    snapshot.last_input_time.insert(player_id, now);
                                    snapshot.player_inputs.insert(player_id, input);
                                    true
                                }
                            } else {
                                // First input - accept and record time
                                snapshot.last_input_time.insert(player_id, now);
                                snapshot.player_inputs.insert(player_id, input);
                                true
                            }
                        };

                        // Log rate limit violations (optional)
                        if !should_accept {
                            // Could log here for debugging
                            // println!("Rate limit exceeded for player {}", player_id);
                        }
                    }
                }
            }

            // Broadcast game state to client
            msg = rx.recv() => {
                if let Ok(text) = msg {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break; // Send failed, connection likely closed
                    }
                }
            }
        }
    }

    // Cleanup on disconnect (single lock)
    {
        let mut snapshot = state.game.lock().await;

        // Remove from connected players
        snapshot.connected_players.remove(&player_id);

        // Remove player's ship from game
        snapshot.game.remove_player(player_id);

        // Clean up rate limit tracking
        snapshot.last_input_time.remove(&player_id);

        // Clean up input buffer
        snapshot.player_inputs.remove(&player_id);
    }

    println!("Player {} disconnected", player_id);
}
