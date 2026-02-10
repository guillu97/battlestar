use battlestar_shared::{ClientInput, GameState};
use std::{
    collections::{HashSet, VecDeque},
    sync::{atomic::AtomicU32, Arc},
};
use tokio::sync::{broadcast, Mutex};

use crate::sim;

pub struct AppState {
    pub broadcaster: broadcast::Sender<String>,
    pub game_state: Arc<Mutex<GameState>>,
    pub input_buffer: Arc<Mutex<VecDeque<ClientInput>>>,
    pub connected_players: Arc<Mutex<HashSet<u32>>>,
    pub next_player_id: AtomicU32,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(256);
        let game_state = Arc::new(Mutex::new(sim::game::new_game_state()));
        let input_buffer = Arc::new(Mutex::new(VecDeque::new()));
        let connected_players = Arc::new(Mutex::new(HashSet::new()));
        let next_player_id = AtomicU32::new(1);

        let state = Arc::new(AppState {
            broadcaster: tx,
            game_state,
            input_buffer,
            connected_players,
            next_player_id,
        });

        sim::spawn_game_loop(state.clone());

        state
    }
}
