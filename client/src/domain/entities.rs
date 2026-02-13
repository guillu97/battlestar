use battlestar_shared::Color as NetColor;
use bevy::prelude::*;
use crate::components::{NetworkedPlayer, NetworkedAsteroid, Player, Ship, Velocity, Thruster, ThrusterOwner, Invincible};
use crate::entities::{build_triangle_mesh, build_thruster_mesh, build_circle_mesh};

/// Spawn a player's local ship
pub fn spawn_local_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec3,
) -> Entity {
    let ship_entity = commands
        .spawn((
            Mesh2d(meshes.add(build_triangle_mesh(25.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.0, 0.545, 0.545)))),
            Transform::from_translation(position),
            Ship,
            Player,
            Velocity::default(),
            Invincible { enabled: false },
        ))
        .id();

    spawn_thruster_for_ship(commands, meshes, materials, ship_entity);

    ship_entity
}

/// Spawn a player's local ship with specific color (from server)
pub fn spawn_local_ship_with_color(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec3,
    velocity: Vec2,
    rotation: Quat,
    color: NetColor,
) -> Entity {
    let bevy_color = Color::srgb(color.r, color.g, color.b);

    let ship_entity = commands
        .spawn((
            Mesh2d(meshes.add(build_triangle_mesh(25.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(bevy_color))),
            Transform::from_translation(position).with_rotation(rotation),
            Ship,
            Player,
            Velocity(velocity),
            Invincible { enabled: false },
        ))
        .id();

    spawn_thruster_for_ship(commands, meshes, materials, ship_entity);

    ship_entity
}

/// Spawn a networked player ship (other players)
pub fn spawn_networked_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    id: u32,
    position: Vec3,
    color: NetColor,
) -> Entity {
    let bevy_color = Color::srgb(color.r, color.g, color.b);

    let ship_entity = commands
        .spawn((
            Mesh2d(meshes.add(build_triangle_mesh(25.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(bevy_color))),
            Transform::from_translation(position),
            NetworkedPlayer { id },
            Velocity::default(),
            Invincible { enabled: false },
        ))
        .id();

    spawn_thruster_for_ship(commands, meshes, materials, ship_entity);

    ship_entity
}

/// Spawn a thruster as child of a ship
fn spawn_thruster_for_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    ship_entity: Entity,
) {
    let thruster_entity = commands
        .spawn((
            Mesh2d(meshes.add(build_thruster_mesh())),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
            Transform::from_translation(Vec3::new(0.0, -28.0, -0.1)),
            Thruster {
                base_length: 12.0,
                max_length: 60.0,
                speed_factor: 0.25,
            },
            ThrusterOwner(ship_entity),
        ))
        .id();

    commands.entity(ship_entity).add_child(thruster_entity);
}

/// Spawn a networked asteroid
pub fn spawn_networked_asteroid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    id: u32,
    position: Vec3,
    radius: f32,
) -> Entity {
    commands
        .spawn((
            Mesh2d(meshes.add(build_circle_mesh(radius, 32))),
            MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)))),
            Transform::from_translation(position),
            NetworkedAsteroid { id },
        ))
        .id()
}
