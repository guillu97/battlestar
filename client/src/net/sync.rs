use battlestar_shared::{Color as NetColor, ServerMessage};
use bevy::prelude::*;

use crate::components::{NetworkedAsteroid, NetworkedPlayer};
use crate::constants::WORLD_LIMIT;

use super::transport::NetworkClient;

#[derive(Resource, Default)]
pub struct PlayerColor {
    pub color: Option<NetColor>,
    pub applied: Option<NetColor>,
}

#[derive(Resource, Default)]
pub struct LocalShipEntity(pub Option<Entity>);

// Calculate wrapped distance accounting for world boundaries
fn wrapped_distance(a: Vec2, b: Vec2) -> f32 {
    let world_size = WORLD_LIMIT * 2.0;

    let mut dx = (a.x - b.x).abs();
    let mut dy = (a.y - b.y).abs();

    // Check if wrapping around is shorter
    if dx > WORLD_LIMIT {
        dx = world_size - dx;
    }
    if dy > WORLD_LIMIT {
        dy = world_size - dy;
    }

    (dx * dx + dy * dy).sqrt()
}

pub fn receive_game_state(
    mut commands: Commands,
    mut client: ResMut<NetworkClient>,
    mut player_color: ResMut<PlayerColor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut existing_ships: Query<
        (Entity, &NetworkedPlayer, &mut Transform, &mut crate::components::Velocity),
        Without<crate::components::Player>,
    >,
    mut local_player: Query<
        (&mut Transform, &mut crate::components::Velocity),
        With<crate::components::Player>,
    >,
    mut existing_asteroids: Query<
        (Entity, &NetworkedAsteroid, &mut Transform),
        (Without<NetworkedPlayer>, Without<crate::components::Player>),
    >,
) {
    if !client.connected {
        return;
    }

    let messages: Vec<String> = {
        if let Ok(mut msgs) = client.messages.lock() {
            msgs.drain(..).collect()
        } else {
            return;
        }
    };

    for msg in messages {
        if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&msg) {
            match server_msg {
                ServerMessage::Welcome { assigned_id } => {
                    info!("Received player ID: {}", assigned_id);
                    client.player_id = assigned_id;
                }
                ServerMessage::GameState(game_state) => {
                    // Track which ships we've seen
                    let mut seen_ids = std::collections::HashSet::new();

                    for server_ship in game_state.ships {
                        seen_ids.insert(server_ship.id);

                        // Update local player from server state (server reconciliation)
                        if server_ship.id == client.player_id {
                            player_color.color = Some(server_ship.color);

                            if let Some((mut transform, mut velocity)) = local_player.iter_mut().next() {
                                let server_pos = Vec2::new(server_ship.position.x, server_ship.position.y);
                                let client_pos = transform.translation.truncate();
                                let distance = wrapped_distance(client_pos, server_pos);

                                // If prediction error is too large, snap to server state
                                if distance > 100.0 {
                                    transform.translation.x = server_ship.position.x;
                                    transform.translation.y = server_ship.position.y;
                                    transform.rotation = Quat::from_rotation_z(server_ship.rotation);
                                    velocity.0.x = server_ship.velocity.x;
                                    velocity.0.y = server_ship.velocity.y;
                                } else {
                                    // Gentle correction towards server state (client-side prediction reconciliation)
                                    let blend = 0.2; // Lower = smoother but more divergence tolerance

                                    // Handle position correction with wrapping
                                    let mut dx = server_ship.position.x - transform.translation.x;
                                    let mut dy = server_ship.position.y - transform.translation.y;

                                    // Correct for wrapping
                                    let world_size = WORLD_LIMIT * 2.0;
                                    if dx.abs() > WORLD_LIMIT {
                                        dx = if dx > 0.0 { dx - world_size } else { dx + world_size };
                                    }
                                    if dy.abs() > WORLD_LIMIT {
                                        dy = if dy > 0.0 { dy - world_size } else { dy + world_size };
                                    }

                                    transform.translation.x += dx * blend;
                                    transform.translation.y += dy * blend;

                                    // Interpolate rotation
                                    let target_quat = Quat::from_rotation_z(server_ship.rotation);
                                    transform.rotation = transform.rotation.slerp(target_quat, blend);

                                    // Blend velocity (affects thruster visuals)
                                    velocity.0.x += (server_ship.velocity.x - velocity.0.x) * blend;
                                    velocity.0.y += (server_ship.velocity.y - velocity.0.y) * blend;
                                }
                            }
                            continue;
                        }

                        // Find or create the ship entity for other players
                        let mut found = false;
                        for (_entity, networked, mut transform, mut velocity) in existing_ships.iter_mut() {
                            if networked.id == server_ship.id {
                                // Interpolate other players for smooth network updates
                                let blend = 0.3; // Slightly lower for remote players to reduce jitter
                                transform.translation.x += (server_ship.position.x - transform.translation.x) * blend;
                                transform.translation.y += (server_ship.position.y - transform.translation.y) * blend;

                                let target_quat = Quat::from_rotation_z(server_ship.rotation);
                                transform.rotation = transform.rotation.slerp(target_quat, blend);

                                // Update velocity for thruster visuals
                                velocity.0.x = server_ship.velocity.x;
                                velocity.0.y = server_ship.velocity.y;

                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // Spawn new networked player ship
                            spawn_networked_ship(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                server_ship.id,
                                Vec3::new(server_ship.position.x, server_ship.position.y, 0.0),
                                server_ship.color,
                            );
                        }
                    }

                    // Remove ships that no longer exist (despawn will handle children)
                    for (entity, networked, _, _) in existing_ships.iter() {
                        if !seen_ids.contains(&networked.id) {
                            commands.entity(entity).despawn();
                        }
                    }

                    // Update asteroids from server
                    let mut seen_asteroid_ids = std::collections::HashSet::new();
                    for server_asteroid in game_state.asteroids {
                        seen_asteroid_ids.insert(server_asteroid.id);

                        // Find or create asteroid entity
                        let mut found = false;
                        for (_entity, networked, mut transform) in existing_asteroids.iter_mut() {
                            if networked.id == server_asteroid.id {
                                // Update position with server authority
                                transform.translation.x = server_asteroid.position.x;
                                transform.translation.y = server_asteroid.position.y;
                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // Spawn new networked asteroid
                            spawn_networked_asteroid(
                                &mut commands,
                                &mut meshes,
                                &mut materials,
                                server_asteroid.id,
                                Vec3::new(server_asteroid.position.x, server_asteroid.position.y, 0.0),
                                server_asteroid.radius,
                            );
                        }
                    }

                    // Remove asteroids that no longer exist
                    for (entity, networked, _) in existing_asteroids.iter() {
                        if !seen_asteroid_ids.contains(&networked.id) {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }
    }
}

pub fn update_local_ship_color(
    mut commands: Commands,
    mut local_ship: ResMut<LocalShipEntity>,
    mut player_color: ResMut<PlayerColor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<
        (&Transform, &crate::components::Velocity, &Children),
        With<crate::components::Player>,
    >,
) {
    let Some(color) = player_color.color else {
        return;
    };

    if player_color.applied == Some(color) {
        return;
    }

    let Some(old_entity) = local_ship.0 else {
        return;
    };

    // Get current state before despawning
    if let Ok((transform, velocity, children)) = player_query.get(old_entity) {
        let position = transform.translation;
        let rotation = transform.rotation;
        let vel = velocity.0;

        // Despawn old ship with children
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        commands.entity(old_entity).despawn();

        // Spawn new ship with correct color
        use crate::components::{Player, Ship, Velocity};

        let ship_entity = commands
            .spawn((
                Mesh2d(meshes.add(crate::entities::build_triangle_mesh(25.0))),
                MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(
                    color.r, color.g, color.b,
                )))),
                Transform::from_translation(position).with_rotation(rotation),
                Ship,
                Player,
                Velocity(vel),
            ))
            .id();

        // Recreate thruster
        use crate::components::{Thruster, ThrusterOwner};
        let thruster_entity = commands
            .spawn((
                Mesh2d(meshes.add(crate::entities::build_thruster_mesh())),
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

        // Update the resource
        local_ship.0 = Some(ship_entity);
        player_color.applied = Some(color);
    }
}

fn spawn_networked_ship(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    id: u32,
    position: Vec3,
    color: NetColor,
) {
    use crate::components::{Thruster, ThrusterOwner, Velocity};

    let bevy_color = Color::srgb(color.r, color.g, color.b);

    let ship_entity = commands
        .spawn((
            Mesh2d(meshes.add(crate::entities::build_triangle_mesh(25.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(bevy_color))),
            Transform::from_translation(position),
            NetworkedPlayer { id },
            Velocity::default(),
        ))
        .id();

    // Create thruster as child
    let thruster_entity = commands
        .spawn((
            Mesh2d(meshes.add(crate::entities::build_thruster_mesh())),
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

fn spawn_networked_asteroid(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    id: u32,
    position: Vec3,
    radius: f32,
) {
    commands.spawn((
        Mesh2d(meshes.add(crate::entities::build_circle_mesh(radius, 32))),
        MeshMaterial2d(materials.add(ColorMaterial::from(Color::srgb(0.5, 0.5, 0.5)))),
        Transform::from_translation(position),
        NetworkedAsteroid { id },
    ));
}
