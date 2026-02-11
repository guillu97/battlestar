use crate::components::{Player, Thruster, ThrusterOwner, Velocity};
use crate::constants::*;
use bevy::prelude::*;

// Wrap position when reaching world boundaries (teleport to other side)
pub fn wrap_position(pos: &mut Vec3) {
    if pos.x > WORLD_LIMIT {
        pos.x = -WORLD_LIMIT;
    } else if pos.x < -WORLD_LIMIT {
        pos.x = WORLD_LIMIT;
    }

    if pos.y > WORLD_LIMIT {
        pos.y = -WORLD_LIMIT;
    } else if pos.y < -WORLD_LIMIT {
        pos.y = WORLD_LIMIT;
    }
}

// Client-side prediction for local player
// Applies physics locally for immediate feedback; server corrections are blended in by sync system
pub fn apply_local_physics(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    player_input: Res<crate::net::PlayerInput>,
    time: Res<Time>,
) {
    let Some((mut transform, mut velocity)) = player_query.iter_mut().next() else {
        return;
    };

    let dt = time.delta().as_secs_f32();

    let thrust = player_input.thrust;
    let rotate = player_input.rotate;

    // Apply rotation - NEGATIVE because D key rotates clockwise
    let rotation_delta = -rotate * ROTATION_SPEED * dt;
    transform.rotate_z(rotation_delta);

    // Extract rotation angle from quaternion
    let (_, _, rotation) = transform.rotation.to_euler(EulerRot::ZYX);

    // Apply thrust in facing direction
    // At rotation=0, ship points UP (Y+)
    velocity.0.x -= thrust * rotation.sin() * THRUST_ACCEL * dt;
    velocity.0.y += thrust * rotation.cos() * THRUST_ACCEL * dt;

    // Apply drag (friction)
    let drag_factor = DRAG.powf(dt * 60.0);
    velocity.0.x *= drag_factor;
    velocity.0.y *= drag_factor;

    // Clamp to max speed
    let speed = velocity.0.length();
    if speed > MAX_SPEED {
        let scale = MAX_SPEED / speed;
        velocity.0.x *= scale;
        velocity.0.y *= scale;
    }

    // Apply velocity
    transform.translation.x += velocity.0.x * dt;
    transform.translation.y += velocity.0.y * dt;

    // Wrap position at world boundaries
    wrap_position(&mut transform.translation);
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
