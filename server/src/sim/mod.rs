pub mod game;
pub mod physics;

use std::{sync::Arc, time::Duration};
use crate::state::AppState;

pub fn spawn_game_loop(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        let mut tick_counter = 0u32;

        loop {
            interval.tick().await;

            // Drain input buffer and apply all inputs atomically
            let inputs = {
                let mut buffer = state.input_buffer.lock().await;
                buffer.drain(..).collect::<Vec<_>>()
            };

            let dt = 1.0 / 20.0; // Match tick rate
            let mut gs = state.game_state.lock().await;

            // Process all queued inputs for this tick
            for input in inputs {
                game::apply_input(&mut gs, input, dt);
            }

            game::step(&mut gs, dt);

            // Broadcast at 60Hz (every tick) for maximum responsiveness
            tick_counter += 1;
            if tick_counter % 1 == 0 {
                if let Ok(payload) = serde_json::to_string(&battlestar_shared::ServerMessage::GameState(gs.clone())) {
                    let _ = state.broadcaster.send(payload);
                }
            }
        }
    });
}
