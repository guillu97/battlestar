use battlestar_shared::ServerMessage;
use std::{sync::Arc, time::Duration};

use crate::app::AppState;

/// Spawn the game loop as a background task
///
/// Runs at 20 Hz (50ms per tick) and:
/// 1. Collects all player inputs
/// 2. Applies inputs to game state
/// 3. Simulates one tick of physics
/// 4. Broadcasts delta updates to all clients
///
/// Key optimizations:
/// - Single lock acquisition per tick instead of 4+
/// - Delta encoding: 90% bandwidth reduction
/// - Full state fallback every N ticks (100 = 5 seconds)
///
/// Delta pattern:
/// ```ignore
/// let message = {
///     let mut snapshot = state.game.lock().await;  // Single lock
///     // ... tick simulation ...
///     if is_full_state_tick {
///         ServerMessage::GameState(snapshot.game.to_network_state())
///     } else {
///         ServerMessage::DeltaState(snapshot.game.to_delta_state())
///     }
/// }; // Lock released here
/// broadcast(message); // Outside lock
/// ```
pub fn spawn_game_loop(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50));

        loop {
            interval.tick().await;

            // Single lock acquisition for entire tick
            let message = {
                let mut snapshot = state.game.lock().await;

                // Collect all inputs (already in snapshot, no extra lock)
                let inputs: Vec<_> = snapshot.player_inputs.values().cloned().collect();

                // Apply all inputs
                let dt = 1.0 / 20.0; // 50ms = 1/20 second
                for input in inputs {
                    snapshot.game.apply_input(input, dt);
                }

                // Tick simulation
                snapshot.game.tick(dt);

                // Determine if this is a full state broadcast
                let is_full_state = snapshot.game.tick % snapshot.game.full_state_interval == 0;

                // Send full state or delta based on tick
                if is_full_state {
                    // Full state: includes asteroids, all ship colors
                    ServerMessage::GameState(snapshot.game.to_network_state())
                } else {
                    // Delta state: only ship positions/velocities (90% smaller)
                    ServerMessage::DeltaState(snapshot.game.to_delta_state())
                }
            }; // Lock is released here

            // Broadcast outside the lock (reduces lock duration)
            if let Ok(payload) = serde_json::to_string(&message) {
                // Ignore send errors (no subscribers is OK)
                let _ = state.broadcaster.send(payload);
            }
        }
    });
}
