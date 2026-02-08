use crate::components::{Thruster, ThrusterOwner, Velocity};
use bevy::prelude::*;

// No client-side physics - server is fully authoritative
// All movement is calculated server-side and applied via network.rs

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
