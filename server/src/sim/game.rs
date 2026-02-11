use battlestar_shared::{Asteroid, ClientInput, Color, GameState, Ship, Vec2};
use rand::RngExt;

use crate::constants::*;
use crate::sim::physics::{distance, wrap_position};

pub fn new_game_state() -> GameState {
    GameState {
        ships: Vec::new(),
        asteroids: vec![
            Asteroid {
                id: 1,
                position: Vec2 { x: 200.0, y: 100.0 },
                velocity: Vec2 { x: 20.0, y: 15.0 },
                radius: 16.0,
            },
            Asteroid {
                id: 2,
                position: Vec2 { x: -150.0, y: -120.0 },
                velocity: Vec2 { x: -10.0, y: 25.0 },
                radius: 24.0,
            },
        ],
        tick: 0,
    }
}

pub fn apply_input(gs: &mut GameState, mut input: ClientInput, dt: f32) {
    // Validate and clamp inputs to prevent cheating
    input.thrust = input.thrust.clamp(-1.0, 1.0);
    input.rotate = input.rotate.clamp(-1.0, 1.0);

    let ship_exists = gs.ships.iter().any(|ship| ship.id == input.player_id);

    if !ship_exists {
        let mut rng = rand::rng();
        gs.ships.push(Ship {
            id: input.player_id,
            position: Vec2 { x: 0.0, y: 0.0 },
            velocity: Vec2 { x: 0.0, y: 0.0 },
            rotation: 0.0,
            color: Color {
                r: rng.random_range(0.3..1.0),
                g: rng.random_range(0.3..1.0),
                b: rng.random_range(0.3..1.0),
            },
        });
    }

    let ship = gs
        .ships
        .iter_mut()
        .find(|ship| ship.id == input.player_id)
        .expect("ship exists");

    // Apply rotation with dt - NEGATIVE because D key should rotate clockwise
    ship.rotation -= input.rotate * ROTATION_SPEED * dt;

    // Apply thrust in facing direction with dt
    // At rotation=0, ship points UP (Y+), not RIGHT
    // So we need: x = -sin(rotation), y = cos(rotation)
    ship.velocity.x -= input.thrust * ship.rotation.sin() * THRUST_ACCEL * dt;
    ship.velocity.y += input.thrust * ship.rotation.cos() * THRUST_ACCEL * dt;
}

pub fn step(gs: &mut GameState, dt: f32) {
    gs.tick = gs.tick.wrapping_add(1);

    // Integrate velocity (1 unit = 1 pixel/second at 60Hz)
    for ship in &mut gs.ships {
        // Apply drag (friction)
        let drag_factor = DRAG.powf(dt * 60.0);
        ship.velocity.x *= drag_factor;
        ship.velocity.y *= drag_factor;

        // Clamp to max speed
        let speed = (ship.velocity.x * ship.velocity.x + ship.velocity.y * ship.velocity.y).sqrt();
        if speed > MAX_SPEED {
            let scale = MAX_SPEED / speed;
            ship.velocity.x *= scale;
            ship.velocity.y *= scale;
        }

        ship.position.x += ship.velocity.x * dt;
        ship.position.y += ship.velocity.y * dt;
        wrap_position(&mut ship.position);
    }

    for asteroid in &mut gs.asteroids {
        asteroid.position.x += asteroid.velocity.x * dt;
        asteroid.position.y += asteroid.velocity.y * dt;
        wrap_position(&mut asteroid.position);
    }

    // Check collisions and destroy ships that hit asteroids
    for ship in &mut gs.ships {
        for asteroid in &gs.asteroids {
            let collision_distance = SHIP_RADIUS + asteroid.radius;
            if distance(ship.position, asteroid.position) < collision_distance {
                // Ship destroyed - respawn at center with zero velocity
                ship.position = Vec2 { x: 0.0, y: 0.0 };
                ship.velocity = Vec2 { x: 0.0, y: 0.0 };
                ship.rotation = 0.0;
            }
        }
    }
}
