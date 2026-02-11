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

            // Read latest input per player (exactly one input per player per tick)
            let inputs: Vec<_> = {
                let inputs = state.player_inputs.lock().await;
                inputs.values().cloned().collect()
            };

            let dt = 1.0 / 20.0; // Match tick rate
            let mut gs = state.game_state.lock().await;

            // Apply latest input for each player
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
