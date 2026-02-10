use battlestar_shared::ClientInput;
use bevy::prelude::*;

use super::transport::{NetworkClient, WebSocketHandle};

pub fn send_player_input(
    client: Res<NetworkClient>,
    ws_handle: Option<Res<WebSocketHandle>>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    if !client.connected || client.player_id == 0 {
        return; // Wait for server to assign ID
    }

    let Some(ws_handle) = ws_handle else {
        return;
    };

    let mut thrust = 0.0;
    let mut rotate = 0.0;

    if kb_input.pressed(KeyCode::KeyW) {
        thrust += 1.0;
    }
    if kb_input.pressed(KeyCode::KeyS) {
        thrust -= 1.0;
    }
    if kb_input.pressed(KeyCode::KeyA) {
        rotate -= 1.0;
    }
    if kb_input.pressed(KeyCode::KeyD) {
        rotate += 1.0;
    }

    // Send raw input values - server handles physics timing
    let input = ClientInput {
        player_id: client.player_id,
        thrust,
        rotate,
    };

    if let Ok(json) = serde_json::to_string(&input) {
        let _ = ws_handle.ws.send_with_str(&json);
    }
}
