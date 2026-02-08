use crate::components::{Asteroid, Ship};
use crate::systems::network::LocalShipEntity;
use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ship_entity = Ship::spawn(&mut commands, &mut meshes, &mut materials, Vec3::ZERO);
    commands.insert_resource(LocalShipEntity(Some(ship_entity)));
    Asteroid::spawn(&mut commands, Vec3::new(120.0, 80.0, 0.0), 20.0);
}

pub fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        Text::new("Move the light with WASD.\nThe camera will smoothly track the light."),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12.0),
            left: px(12.0),
            ..default()
        },
    ));
}
