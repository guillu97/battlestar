// Core modules
pub mod math;
pub mod physics;
pub mod entities;
pub mod input;
pub mod network;

// Re-export commonly used types
pub use math::Vec2;
pub use physics::{PhysicsConstants, Input};
pub use entities::{Ship, Asteroid, Color};
pub use network::{ServerMessage, ClientInput, GameState, DeltaState, ShipUpdate};
