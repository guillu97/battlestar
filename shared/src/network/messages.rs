use serde::{Deserialize, Serialize};
use crate::entities::{Ship, Asteroid};
use super::delta::DeltaState;

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { assigned_id: u32 },
    GameState(GameState),
    DeltaState(DeltaState),
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInput {
    pub player_id: u32,
    pub thrust: f32,
    pub rotate: f32,
}

/// Full game state sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub ships: Vec<Ship>,
    pub asteroids: Vec<Asteroid>,
    pub tick: u64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            ships: Vec::new(),
            asteroids: Vec::new(),
            tick: 0,
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
