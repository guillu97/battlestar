use battlestar_shared::{ClientInput, GameState};
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU32, Arc},
    time::Instant,
};
use tokio::sync::{broadcast, Mutex};

use crate::sim;

pub struct AppState {
    pub broadcaster: broadcast::Sender<String>,
    pub game_state: Arc<Mutex<GameState>>,
    pub player_inputs: Arc<Mutex<HashMap<u32, ClientInput>>>,
    pub connected_players: Arc<Mutex<HashSet<u32>>>,
    pub next_player_id: AtomicU32,
    // Rate limiting: track last input time per player
    pub last_input_time: Arc<Mutex<HashMap<u32, Instant>>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(256);
        let game_state = Arc::new(Mutex::new(sim::game::new_game_state()));
        let player_inputs = Arc::new(Mutex::new(HashMap::new()));
        let connected_players = Arc::new(Mutex::new(HashSet::new()));
        let next_player_id = AtomicU32::new(1);
        let last_input_time = Arc::new(Mutex::new(HashMap::new()));

        let state = Arc::new(AppState {
            broadcaster: tx,
            game_state,
            player_inputs,
            connected_players,
            next_player_id,
            last_input_time,
        });

        sim::spawn_game_loop(state.clone());

        state
    }
}
