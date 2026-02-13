use bevy::prelude::*;

mod components;
mod constants;
mod domain;
mod entities;
mod net;
mod systems;

// src/lib.rs (extrait minimal pour Bevy WASM)
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    build_app().run();
}

fn build_app() -> App {
    let mut app = App::new();
    
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                canvas: Some("#canvas".to_string()), // ou laisse default
                ..default()
            }),
            ..default()
        }))
        .add_systems(
            Startup,
            (
                systems::setup::setup,
                systems::setup::setup_instructions,
                systems::camera::setup_camera,
                systems::joystick::setup_joystick,
                net::setup_network,
            ),
        )
        .add_systems(
            Update,
            (
                net::poll_connection_state,
                systems::joystick::update_joystick,
                net::gather_player_input
                    .after(systems::joystick::update_joystick),
                net::send_player_input
                    .after(net::gather_player_input),
                net::receive_game_state,
                systems::movement::apply_local_physics
                    .after(net::gather_player_input)
                    .after(net::receive_game_state),  // CRUCIAL: Apply local physics AFTER server updates
                systems::movement::update_asteroids,  // Update asteroid positions locally
                systems::movement::update_thruster_length
                    .after(systems::movement::apply_local_physics),
                systems::camera::update_camera
                    .after(systems::movement::apply_local_physics),
                net::update_local_ship_color,
                systems::invincibility::blink_invincible_ships,  // Blink effect for invincible ships
            ),
        )
        .insert_resource(net::PlayerInput::default())
        .insert_resource(net::PlayerColor::default())
        .insert_resource(net::LocalShipEntity::default())
        .insert_resource(net::InputThrottle::default());
    app
}


