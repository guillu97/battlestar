use battlestar_shared::ClientInput;
use bevy::prelude::*;

use crate::components::Joystick;
use super::transport::{NetworkClient, WebSocketHandle};

/// Shared input state gathered each frame from keyboard + joystick.
/// Read by both the network sender and the local physics prediction.
#[derive(Resource, Default)]
pub struct PlayerInput {
    pub thrust: f32,
    pub rotate: f32,
}

/// Throttle resource to limit input send rate to server
#[derive(Resource)]
pub struct InputThrottle {
    timer: Timer,
}

impl Default for InputThrottle {
    fn default() -> Self {
        Self {
            // Send inputs at ~60Hz for responsive server updates
            // Server rate-limits at 66Hz for anti-cheat
            timer: Timer::from_seconds(0.016, TimerMode::Repeating),
        }
    }
}

/// Gather input from keyboard and mobile joystick into shared PlayerInput resource
pub fn gather_player_input(
    kb_input: Res<ButtonInput<KeyCode>>,
    joystick_query: Query<&Joystick>,
    mut player_input: ResMut<PlayerInput>,
) {
    let mut thrust = 0.0;
    let mut rotate = 0.0;

    // Keyboard
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

    // Mobile joystick — takes priority when active
    if let Some(joystick) = joystick_query.iter().next() {
        if joystick.input.length() > 0.1 {
            thrust = joystick.input.y;
            rotate = joystick.input.x;
        }
    }

    player_input.thrust = thrust;
    player_input.rotate = rotate;
}

/// Send current input to server at throttled rate
pub fn send_player_input(
    client: Res<NetworkClient>,
    ws_handle: Option<Res<WebSocketHandle>>,
    player_input: Res<PlayerInput>,
    mut throttle: ResMut<InputThrottle>,
    time: Res<Time>,
) {
    if !client.connected || client.player_id == 0 {
        return;
    }

    let Some(ws_handle) = ws_handle else {
        return;
    };

    throttle.timer.tick(time.delta());
    if !throttle.timer.just_finished() {
        return;
    }

    let input = ClientInput {
        player_id: client.player_id,
        thrust: player_input.thrust,
        rotate: player_input.rotate,
    };

    if let Ok(json) = serde_json::to_string(&input) {
        let _ = ws_handle.ws.send_with_str(&json);
    }
}
