use crate::components::{Player, Thruster, ThrusterOwner, Velocity};
use crate::constants::{ANGULAR_SPEED, PLAYER_SPEED};
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn move_player(
    mut player: Single<(&mut Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction.length_squared() > 0.0 {
        let target = direction.y.atan2(direction.x) - PI / 2.0;
        let current = player.0.rotation.to_euler(EulerRot::XYZ).2;
        let delta = (target - current + PI).rem_euclid(2.0 * PI) - PI;
        let max_step = ANGULAR_SPEED * time.delta_secs();
        let step = delta.clamp(-max_step, max_step);
        player.0.rotation = Quat::from_rotation_z(current + step);
    }

    let velocity = direction.normalize_or_zero() * PLAYER_SPEED;
    let move_delta = velocity * time.delta_secs();
    player.0.translation += move_delta.extend(0.0);
    player.1.0 = velocity;
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
