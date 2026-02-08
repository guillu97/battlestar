use crate::game::{ClientInput, GameState, ServerMessage};
use std::{
    collections::{HashSet, VecDeque},
    sync::{atomic::AtomicU32, Arc},
    time::Duration,
};
use tokio::sync::{broadcast, Mutex};

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
        let game_state = Arc::new(Mutex::new(GameState::new()));
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

        spawn_game_loop(state.clone());

        state
    }
}

fn spawn_game_loop(state: Arc<AppState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(50)); 
        let mut tick_counter = 0u32;
        
        loop {
            interval.tick().await;
            
            // Drain input buffer and apply all inputs atomically
            let inputs: Vec<ClientInput> = {
                let mut buffer = state.input_buffer.lock().await;
                buffer.drain(..).collect()
            };
            
            let dt = 1.0 / 20.0; // Match tick rate
            let mut gs = state.game_state.lock().await;
            
            // Process all queued inputs for this tick
            for input in inputs {
                gs.apply_input(input, dt);
            }
            
            gs.step(dt);
            
            // Broadcast at 60Hz (every tick) for maximum responsiveness
            tick_counter += 1;
            if tick_counter % 1 == 0 {
                if let Ok(payload) = serde_json::to_string(&ServerMessage::GameState(gs.clone())) {
                    let _ = state.broadcaster.send(payload);
                }
            }
        }
    });
}
