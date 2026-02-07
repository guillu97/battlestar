use bevy::prelude::Component;
use bevy::prelude::Vec2;

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
