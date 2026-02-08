use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
