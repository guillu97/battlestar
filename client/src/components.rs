use bevy::prelude::Component;
use bevy::prelude::Vec2;
use bevy::prelude::Bundle;

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
