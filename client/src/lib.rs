use bevy::prelude::*;

mod components;
mod constants;
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
                systems::movement::apply_local_physics,
                systems::movement::update_thruster_length,
                systems::camera::update_camera,
                systems::joystick::update_joystick,
            ),
        )
        .add_systems(
            Update,
            (
                net::send_player_input,
                net::receive_game_state,
            ),
        )
        .add_systems(Update, net::update_local_ship_color)
        .insert_resource(net::PlayerColor::default())
        .insert_resource(net::LocalShipEntity::default());
    app
}


