use bevy::prelude::Component;
use bevy::prelude::Vec2;
use bevy::prelude::Bundle;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Thruster {
	pub base_length: f32,
	pub max_length: f32,
	pub speed_factor: f32,
}

#[derive(Component)]
pub struct ThrusterOwner(pub bevy::prelude::Entity);

#[derive(Component)]
pub struct Joystick {
	pub input: Vec2,
}

#[derive(Component)]
pub struct JoystickKnob;

#[derive(Bundle)]
pub struct JoystickBundle {
	pub joystick: Joystick,
	pub node: bevy::prelude::Node,
	pub background_color: bevy::prelude::BackgroundColor,
	pub transform: bevy::prelude::Transform,
	pub global_transform: bevy::prelude::GlobalTransform,
}

#[derive(Bundle)]
pub struct JoystickKnobBundle {
	pub knob: JoystickKnob,
	pub node: bevy::prelude::Node,
	pub background_color: bevy::prelude::BackgroundColor,
	pub transform: bevy::prelude::Transform,
	pub global_transform: bevy::prelude::GlobalTransform,
}

// Network-related structures matching server
#[derive(Component)]
pub struct NetworkedPlayer {
	pub id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInput {
	pub player_id: u32,
	pub thrust: f32,
	pub rotate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
	pub ships: Vec<ServerShip>,
	pub asteroids: Vec<ServerAsteroid>,
	pub tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerShip {
	pub id: u32,
	pub position: ServerVec2,
	pub velocity: ServerVec2,
	pub rotation: f32,
	pub color: ServerColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerAsteroid {
	pub id: u32,
	pub position: ServerVec2,
	pub velocity: ServerVec2,
	pub radius: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ServerVec2 {
	pub x: f32,
	pub y: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ServerColor {
	pub r: f32,
	pub g: f32,
	pub b: f32,
}
