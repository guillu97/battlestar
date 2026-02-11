use battlestar_shared::{physics, Input, PhysicsConstants, Vec2};
use crate::components::{Player, Thruster, ThrusterOwner, Velocity};
use crate::constants::*;
use bevy::prelude::*;

// Client-side prediction for local player
// Applies physics locally for immediate feedback; server corrections are blended in by sync system
// Uses shared physics engine to ensure identical behavior with server
pub fn apply_local_physics(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    player_input: Res<crate::net::PlayerInput>,
    time: Res<Time>,
) {
    let Some((mut transform, mut velocity)) = player_query.iter_mut().next() else {
        return;
    };

    let dt = time.delta().as_secs_f32();

    // Create physics constants from game constants
    let constants = PhysicsConstants::from_game_constants(
        THRUST_ACCEL,
        ROTATION_SPEED,
        MAX_SPEED,
        DRAG,
        WORLD_LIMIT,
        SHIP_RADIUS,
    );

    // Create shared input structure
    let input = Input::new(player_input.thrust, player_input.rotate);

    // Convert Bevy types to shared types
    let mut position = Vec2::new(transform.translation.x, transform.translation.y);
    let mut vel = Vec2::new(velocity.0.x, velocity.0.y);

    // Extract Z rotation from quaternion (for 2D game)
    // Direct calculation: for a Z-axis rotation quaternion
    let mut rotation = 2.0 * transform.rotation.z.atan2(transform.rotation.w);

    // DEBUG: Log rotation values
    if player_input.rotate != 0.0 {
        info!("🔄 Input rotate: {}, rotation before: {:.2}, rotation_speed: {}, dt: {}",
              player_input.rotate, rotation, constants.rotation_speed, dt);
    }

    // Apply shared physics (same code as server!)
    physics::apply_ship_physics(
        &mut position,
        &mut vel,
        &mut rotation,
        &input,
        dt,
        &constants,
    );

    // DEBUG: Log rotation after physics
    if player_input.rotate != 0.0 {
        info!("🔄 Rotation after physics: {:.2}", rotation);
    }

    // Convert back to Bevy types
    transform.translation.x = position.x;
    transform.translation.y = position.y;
    transform.rotation = Quat::from_rotation_z(rotation);
    velocity.0.x = vel.x;
    velocity.0.y = vel.y;
}

pub fn update_thruster_length(
    mut thrusters: Query<(&Thruster, &ThrusterOwner, &mut Transform)>,
    velocities: Query<&Velocity>,
) {
    for (thruster, owner, mut transform) in &mut thrusters {
        let Ok(velocity) = velocities.get(owner.0) else {
            continue;
        };

        let speed = velocity.0.length();
        let length = (thruster.base_length + speed * thruster.speed_factor)
            .min(thruster.max_length);
        transform.scale.y = length / thruster.base_length;
    }
}
