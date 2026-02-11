pub mod ship;
pub mod asteroid;

pub use ship::Ship;
pub use asteroid::Asteroid;

use serde::{Deserialize, Serialize

};

/// Color representation for entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0 };

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}
