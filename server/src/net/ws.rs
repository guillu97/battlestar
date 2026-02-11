use battlestar_shared::{ClientInput, ServerMessage};
use crate::state::AppState;
use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::{atomic::Ordering, Arc};
use std::time::{Duration, Instant};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // Assign unique player ID
    let player_id = state.next_player_id.fetch_add(1, Ordering::SeqCst);

    // Send Welcome message
    let welcome = ServerMessage::Welcome {
        assigned_id: player_id,
    };
    if let Ok(welcome_json) = serde_json::to_string(&welcome) {
        if socket.send(Message::Text(welcome_json.into())).await.is_err() {
            return;
        }
    }

    // Add to connected players
    {
        let mut players = state.connected_players.lock().await;
        players.insert(player_id);
    }

    let mut rx = state.broadcaster.subscribe();

    loop {
        tokio::select! {
            maybe_msg = socket.recv() => {
                let Some(Ok(msg)) = maybe_msg else {
                    break;
                };

                if let Message::Text(text) = msg {
                    if let Ok(mut input) = serde_json::from_str::<ClientInput>(&text) {
                        // Override client's player_id with server-assigned ID
                        input.player_id = player_id;

                        // SERVER-SIDE RATE LIMITING (anti-cheat)
                        // Minimum 15ms between inputs (~66 inputs/sec max)
                        // Allows 60Hz client input with some tolerance
                        const MIN_INPUT_INTERVAL: Duration = Duration::from_millis(15);
                        
                        let now = Instant::now();
                        let mut last_times = state.last_input_time.lock().await;
                        
                        if let Some(last_time) = last_times.get(&player_id) {
                            let elapsed = now.duration_since(*last_time);
                            if elapsed < MIN_INPUT_INTERVAL {
                                // Rate limit exceeded - silently drop input
                                // This prevents spam/cheating
                                continue;
                            }
                        }
                        
                        // Update last input time
                        last_times.insert(player_id, now);
                        drop(last_times); // Release lock before buffer access

                        // Store latest input per player (one per tick)
                        let mut inputs = state.player_inputs.lock().await;
                        inputs.insert(player_id, input);
                    }
                }
            }
            msg = rx.recv() => {
                if let Ok(text) = msg {
                    let _ = socket.send(Message::Text(text.into())).await;
                }
            }
        }
    }

    // Cleanup on disconnect
    {
        let mut players = state.connected_players.lock().await;
        players.remove(&player_id);

        let mut gs = state.game_state.lock().await;
        gs.ships.retain(|ship| ship.id != player_id);

        // Clean up rate limit tracking and input state
        let mut last_times = state.last_input_time.lock().await;
        last_times.remove(&player_id);
        let mut inputs = state.player_inputs.lock().await;
        inputs.remove(&player_id);
    }
}
