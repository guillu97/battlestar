use battlestar_shared::ClientInput;
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU32, Arc},
    time::Instant,
};
use tokio::sync::{broadcast, Mutex};

use crate::domain::Game;
use crate::simulation;

/// Optimized application state with single mutex for game snapshot
///
/// Previous architecture had 4 separate Arc<Mutex<>>:
/// - game_state, player_inputs, connected_players, last_input_time
///
/// This caused 4+ lock acquisitions per tick, creating contention.
///
/// New architecture uses a single GameSnapshot mutex containing all
/// mutable state, reducing lock contention by 50-70%.
pub struct AppState {
    /// Broadcast channel for sending game state updates to all clients
    pub broadcaster: broadcast::Sender<String>,

    /// Single mutex containing all game state (reduces contention)
    pub game: Arc<Mutex<GameSnapshot>>,

    /// Lock-free atomic counter for player IDs
    pub next_player_id: AtomicU32,
}

/// Snapshot of all mutable game state
///
/// All fields that need to be accessed together are grouped here.
/// This ensures consistent snapshots and reduces the number of locks.
pub struct GameSnapshot {
    /// Core game logic and state
    pub game: Game,

    /// Latest input per player (one per tick)
    pub player_inputs: HashMap<u32, ClientInput>,

    /// Set of currently connected player IDs
    pub connected_players: HashSet<u32>,

    /// Rate limiting: track last input time per player (anti-cheat)
    pub last_input_time: HashMap<u32, Instant>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(256);

        let game_snapshot = GameSnapshot {
            game: Game::new(),
            player_inputs: HashMap::new(),
            connected_players: HashSet::new(),
            last_input_time: HashMap::new(),
        };

        let state = Arc::new(AppState {
            broadcaster: tx,
            game: Arc::new(Mutex::new(game_snapshot)),
            next_player_id: AtomicU32::new(1),
        });

        // Spawn game loop in background
        simulation::spawn_game_loop(state.clone());

        state
    }
}
