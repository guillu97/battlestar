use bevy::prelude::{Bundle, Component, Vec2};

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MapBoundary;

// Velocity is read-only from server updates (no client-side physics)
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

// Networked entities
#[derive(Component)]
pub struct NetworkedPlayer {
    pub id: u32,
}

#[derive(Component)]
pub struct NetworkedAsteroid {
    pub id: u32,
}

// Invincibility tracking for ships after respawn
#[derive(Component)]
pub struct Invincible {
    pub enabled: bool,
}
