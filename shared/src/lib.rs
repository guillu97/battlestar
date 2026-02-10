use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { assigned_id: u32 },
    GameState(GameState),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInput {
    pub player_id: u32,
    pub thrust: f32,
    pub rotate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub ships: Vec<Ship>,
    pub asteroids: Vec<Asteroid>,
    pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asteroid {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
