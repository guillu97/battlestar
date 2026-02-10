use battlestar_shared::{ClientInput, ServerMessage};
use crate::state::AppState;
use axum::{
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use std::sync::{atomic::Ordering, Arc};

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

                        // Push input to buffer to be processed in game loop
                        let mut buffer = state.input_buffer.lock().await;
                        buffer.push_back(input);
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
    }
}
