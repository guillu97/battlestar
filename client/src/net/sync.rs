use battlestar_shared::{Color as NetColor, ServerMessage};
use bevy::prelude::*;

use crate::components::{NetworkedAsteroid, NetworkedPlayer, Invincible};
use crate::domain;

use super::transport::NetworkClient;

#[derive(Resource, Default)]
pub struct PlayerColor {
    pub color: Option<NetColor>,
    pub applied: Option<NetColor>,
}

#[derive(Resource, Default)]
pub struct LocalShipEntity(pub Option<Entity>);

pub fn receive_game_state(
    mut commands: Commands,
    mut client: ResMut<NetworkClient>,
    mut player_color: ResMut<PlayerColor>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut existing_ships: Query<
        (Entity, &NetworkedPlayer, &mut Transform, &mut crate::components::Velocity, &mut Invincible),
        Without<crate::components::Player>,
    >,
    mut local_player: Query<
        (&mut Transform, &mut crate::components::Velocity, &mut Invincible),
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
                ServerMessage::DeltaState(delta_state) => {
                    // Delta update: only changed ships (90% bandwidth reduction)
                    // Track removed ships if this is a full state delta
                    let mut seen_ids = if delta_state.is_full_state {
                        Some(std::collections::HashSet::new())
                    } else {
                        None
                    };

                    for ship_update in delta_state.changed_ships {
                        if let Some(ref mut ids) = seen_ids {
                            ids.insert(ship_update.id);
                        }

                        // Update local player from server state (server-authoritative)
                        if ship_update.id == client.player_id {
                            // Update color if provided (spawn/respawn)
                            if let Some(color) = ship_update.color {
                                player_color.color = Some(color);
                            }

                            if let Some((mut transform, mut velocity, mut invincible)) = local_player.iter_mut().next() {
                                let server_pos = Vec3::new(
                                    ship_update.position.x,
                                    ship_update.position.y,
                                    0.0,
                                );

                                // Update invincibility state
                                invincible.enabled = ship_update.is_invincible.unwrap_or(false);

                                // If far from server (collision/respawn), snap immediately
                                // Otherwise smooth correction to avoid jitter with prediction
                                let distance = transform.translation.distance(server_pos);
                                let blend = if distance > 100.0 { 1.0 } else { 0.3 };

                                transform.translation = transform.translation.lerp(server_pos, blend);

                                // For rotation: only reconcile if there's a big discrepancy (collision/respawn)
                                // Otherwise trust client-side prediction to avoid fighting with local input
                                if distance > 100.0 {
                                    let server_rot = Quat::from_rotation_z(ship_update.rotation);
                                    transform.rotation = server_rot;
                                }

                                // Snap velocity so client prediction stays accurate
                                velocity.0.x = ship_update.velocity.x;
                                velocity.0.y = ship_update.velocity.y;
                            }
                            continue;
                        }

                        // Find or create the ship entity for other players
                        let mut found = false;
                        for (_entity, networked, mut transform, mut velocity, mut invincible) in existing_ships.iter_mut() {
                            if networked.id == ship_update.id {
                                // Update invincibility state
                                invincible.enabled = ship_update.is_invincible.unwrap_or(false);

                                // Interpolate other players for smooth network updates
                                let blend = 0.3;
                                transform.translation.x += (ship_update.position.x - transform.translation.x) * blend;
                                transform.translation.y += (ship_update.position.y - transform.translation.y) * blend;

                                let target_quat = Quat::from_rotation_z(ship_update.rotation);
                                transform.rotation = transform.rotation.slerp(target_quat, blend);

                                // Update velocity for thruster visuals
                                velocity.0.x = ship_update.velocity.x;
                                velocity.0.y = ship_update.velocity.y;

                                found = true;
                                break;
                            }
                        }

                        if !found {
                            // Need color to spawn - skip if not provided
                            if let Some(color) = ship_update.color {
                                domain::spawn_networked_ship(
                                    &mut commands,
                                    &mut meshes,
                                    &mut materials,
                                    ship_update.id,
                                    Vec3::new(ship_update.position.x, ship_update.position.y, 0.0),
                                    color,
                                );
                            }
                        }
                    }

                    // Remove ships that were explicitly removed
                    for removed_id in delta_state.removed_ship_ids {
                        for (entity, networked, _, _, _) in existing_ships.iter() {
                            if networked.id == removed_id {
                                commands.entity(entity).despawn();
                                break;
                            }
                        }
                    }

                    // If full state delta, remove ships not in update
                    if let Some(seen_ids) = seen_ids {
                        for (entity, networked, _, _, _) in existing_ships.iter() {
                            if !seen_ids.contains(&networked.id) {
                                commands.entity(entity).despawn();
                            }
                        }
                    }
                }
                ServerMessage::GameState(game_state) => {
                    // Track which ships we've seen
                    let mut seen_ids = std::collections::HashSet::new();

                    for server_ship in game_state.ships {
                        seen_ids.insert(server_ship.id);

                        // Update local player from server state (server-authoritative)
                        if server_ship.id == client.player_id {
                            player_color.color = Some(server_ship.color);

                            if let Some((mut transform, mut velocity, mut invincible)) = local_player.iter_mut().next() {
                                let server_pos = Vec3::new(
                                    server_ship.position.x,
                                    server_ship.position.y,
                                    0.0,
                                );

                                // Default to not invincible for GameState (no field in Ship)
                                invincible.enabled = false;

                                // If far from server (collision/respawn), snap immediately
                                // Otherwise smooth correction to avoid jitter with prediction
                                let distance = transform.translation.distance(server_pos);
                                let blend = if distance > 100.0 { 1.0 } else { 0.3 };

                                transform.translation = transform.translation.lerp(server_pos, blend);

                                // For rotation: only reconcile if there's a big discrepancy (collision/respawn)
                                // Otherwise trust client-side prediction to avoid fighting with local input
                                if distance > 100.0 {
                                    let server_rot = Quat::from_rotation_z(server_ship.rotation);
                                    transform.rotation = server_rot;
                                }

                                // Snap velocity so client prediction stays accurate
                                velocity.0.x = server_ship.velocity.x;
                                velocity.0.y = server_ship.velocity.y;
                            }
                            continue;
                        }

                        // Find or create the ship entity for other players
                        let mut found = false;
                        for (_entity, networked, mut transform, mut velocity, mut invincible) in existing_ships.iter_mut() {
                            if networked.id == server_ship.id {
                                // Default to not invincible for GameState
                                invincible.enabled = false;

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
                            domain::spawn_networked_ship(
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
                    for (entity, networked, _, _, _) in existing_ships.iter() {
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
                            domain::spawn_networked_asteroid(
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
                Invincible { enabled: false },
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
            Invincible { enabled: false },
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
