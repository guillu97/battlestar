pub mod movement;
pub mod collision;

pub use movement::*;
pub use collision::*;

/// Physics constants for game simulation
#[derive(Debug, Clone, Copy)]
pub struct PhysicsConstants {
    pub thrust_accel: f32,
    pub rotation_speed: f32,
    pub max_speed: f32,
    pub drag: f32,
    pub world_limit: f32,
    pub ship_radius: f32,
}

impl PhysicsConstants {
    pub fn from_game_constants(
        thrust_accel: f32,
        rotation_speed: f32,
        max_speed: f32,
        drag: f32,
        world_limit: f32,
        ship_radius: f32,
    ) -> Self {
        Self {
            thrust_accel,
            rotation_speed,
            max_speed,
            drag,
            world_limit,
            ship_radius,
        }
    }
}
