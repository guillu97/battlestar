use bevy::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;

mod components;
mod constants;
mod entities;
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
        .add_plugins(ShapePlugin)
        .add_systems(
            Startup,
            (
                systems::setup::setup,
                systems::setup::setup_instructions,
                systems::camera::setup_camera,
            ),
        )
        .add_systems(
            Update,
            (
                systems::movement::move_player,
                systems::movement::update_thruster_length,
                systems::camera::update_camera,
            ),
        );
    app
}


