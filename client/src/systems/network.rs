use crate::components::{ClientInput, GameState, NetworkedPlayer, ServerColor};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[derive(Resource)]
pub struct NetworkClient {
    pub player_id: u32,
    pub messages: Arc<Mutex<Vec<String>>>,
    pub connected: bool,
    // Store WebSocket URL instead of WebSocket itself (WebSocket is not Send+Sync)
    ws_url: String,
}

#[derive(Resource, Default)]
pub struct PlayerColor {
    pub color: Option<ServerColor>,
    pub applied: Option<ServerColor>,
}

#[derive(Resource, Default)]
pub struct LocalShipEntity(pub Option<Entity>);

// Manual Send+Sync implementation
// Safe because we only use thread-safe types (Arc<Mutex<_>>)
unsafe impl Send for NetworkClient {}
unsafe impl Sync for NetworkClient {}

impl Default for NetworkClient {
    fn default() -> Self {
        let player_id = (js_sys::Math::random() * 1000000.0) as u32;
        
        // Determine WebSocket URL
        let ws_url = {
            let window = web_sys::window().expect("no global `window` exists");
            let location = window.location();
            
            // Get hostname - if it's local development, use localhost:3000
            let hostname = location.hostname().unwrap_or_default();
            let is_local = hostname.is_empty() 
                || hostname == "localhost" 
                || hostname == "127.0.0.1"
                || hostname.starts_with("localhost.")
                || hostname.starts_with("127.0.0.1");
            
            let protocol = location.protocol().unwrap_or_else(|_| "http:".to_string());
            let ws_protocol = if protocol == "https:" { "wss" } else { "ws" };
            
            let ws_url = if is_local {
                // Force localhost:3000 for local development
                format!("{}://localhost:3000/ws", ws_protocol)
            } else {
                // For production, use the current host
                let host = location.host().unwrap_or_else(|_| "localhost:3000".to_string());
                format!("{}://{}/ws", ws_protocol, host)
            };
            
            ws_url
        };

        Self {
            player_id,
            messages: Arc::new(Mutex::new(Vec::new())),
            connected: false,
            ws_url,
        }
    }
}

#[derive(Resource)]
pub struct WebSocketHandle {
    ws: WebSocket,
}

// Manual Send+Sync because we're in WASM (single-threaded)
unsafe impl Send for WebSocketHandle {}
unsafe impl Sync for WebSocketHandle {}

pub fn setup_network(mut commands: Commands) {
    let mut client = NetworkClient::default();
    
    let ws_url = client.ws_url.clone();
    info!("Connecting to WebSocket: {}", ws_url);

    match WebSocket::new(&ws_url) {
        Ok(ws) => {
            let messages = client.messages.clone();
            
            // Setup onmessage callback
            let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
                if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                    let msg = String::from(txt);
                    if let Ok(mut msgs) = messages.lock() {
                        msgs.push(msg);
                    }
                }
            });
            ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            onmessage_callback.forget();

            // Setup onerror callback
            let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
                error!("WebSocket error: {:?}", e);
            });
            ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Setup onopen callback
            let onopen_callback = Closure::<dyn FnMut()>::new(move || {
                info!("WebSocket connected!");
            });
            ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            onopen_callback.forget();

            client.connected = true;
            
            commands.insert_resource(WebSocketHandle { ws });
        }
        Err(e) => {
            error!("Failed to create WebSocket: {:?}", e);
        }
    }

    commands.insert_resource(client);
    commands.insert_resource(PlayerColor::default());
    commands.insert_resource(LocalShipEntity::default());
}

pub fn send_player_input(
    client: Res<NetworkClient>,
    ws_handle: Option<Res<WebSocketHandle>>,
    kb_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if !client.connected {
        return;
    }

    let Some(ws_handle) = ws_handle else {
        return;
    };

    let mut thrust = 0.0;
    let mut rotate = 0.0;

    if kb_input.pressed(KeyCode::KeyW) {
        thrust += 1.0;
    }
    if kb_input.pressed(KeyCode::KeyS) {
        thrust -= 1.0;
    }
    if kb_input.pressed(KeyCode::KeyA) {
        rotate -= 1.0;
    }
    if kb_input.pressed(KeyCode::KeyD) {
        rotate += 1.0;
    }

    // Send input every frame (even if zero) so server knows player is still active
    let input = ClientInput {
        player_id: client.player_id,
        thrust,
        rotate: rotate * time.delta_secs() * 3.0,
    };

    if let Ok(json) = serde_json::to_string(&input) {
        let _ = ws_handle.ws.send_with_str(&json);
    }
}

pub fn receive_game_state(
    mut commands: Commands,
    client: Res<NetworkClient>,
    mut player_color: ResMut<PlayerColor>,
    mut existing_ships: Query<(Entity, &NetworkedPlayer, &mut Transform), Without<crate::components::Player>>,
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
        if let Ok(game_state) = serde_json::from_str::<GameState>(&msg) {
            // Track which ships we've seen
            let mut seen_ids = std::collections::HashSet::new();

            for server_ship in game_state.ships {
                seen_ids.insert(server_ship.id);

                // Store color for local player from server
                if server_ship.id == client.player_id {
                    player_color.color = Some(server_ship.color);
                    continue;
                }

                // Find or create the ship entity for other players
                let mut found = false;
                for (_entity, networked, mut transform) in existing_ships.iter_mut() {
                    if networked.id == server_ship.id {
                        // Update existing ship
                        transform.translation.x = server_ship.position.x;
                        transform.translation.y = server_ship.position.y;
                        transform.rotation = Quat::from_rotation_z(server_ship.rotation);
                        found = true;
                        break;
                    }
                }

                if !found {
                    // Spawn new networked player ship
                    spawn_networked_ship(
                        &mut commands,
                        server_ship.id,
                        Vec3::new(server_ship.position.x, server_ship.position.y, 0.0),
                        server_ship.color,
                    );
                }
            }

            // Remove ships that no longer exist
            for (entity, networked, _) in existing_ships.iter() {
                if !seen_ids.contains(&networked.id) {
                    commands.entity(entity).despawn();
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
    player_query: Query<(&Transform, &crate::components::Velocity, &Children), With<crate::components::Player>>,
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
        use crate::components::{Ship, Player, Velocity};
        use bevy_prototype_lyon::prelude::*;
        use bevy::color::palettes::css::BLACK;
        
        let ship_shape = shapes::RegularPolygon {
            sides: 3,
            feature: shapes::RegularPolygonFeature::Radius(25.0),
            ..Default::default()
        };

        let ship_entity = commands
            .spawn((
                ShapeBuilder::with(&ship_shape)
                    .fill(Fill::color(Color::srgb(color.r, color.g, color.b)))
                    .stroke(Stroke::new(BLACK, 2.0f32))
                    .build(),
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
    id: u32,
    position: Vec3,
    color: ServerColor,
) {
    use bevy::color::palettes::css::BLACK;
    use bevy_prototype_lyon::prelude::*;

    let ship_shape = shapes::RegularPolygon {
        sides: 3,
        feature: shapes::RegularPolygonFeature::Radius(25.0),
        ..Default::default()
    };

    let bevy_color = Color::srgb(color.r, color.g, color.b);

    commands.spawn((
        ShapeBuilder::with(&ship_shape)
            .fill(Fill::color(bevy_color))
            .stroke(Stroke::new(BLACK, 2.0f32))
            .build(),
        Transform::from_translation(position),
        NetworkedPlayer { id },
    ));
}
