use bevy::prelude::*;
use crate::components::Invincible;

/// Blink invincible ships by modulating their opacity
///
/// Ships that recently respawned are invincible for 1 second and should
/// visually indicate this by pulsing their opacity at 3 Hz (3 times per second).
/// This makes it clear to all players when a ship is invincible.
pub fn blink_invincible_ships(
    time: Res<Time>,
    query: Query<(&Invincible, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (invincible, material_handle) in query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if invincible.enabled {
                // Blink at 3 Hz (3 times per second) using a sine wave
                let blink_frequency = 3.0;
                let phase = time.elapsed_secs() * blink_frequency * std::f32::consts::TAU;

                // Oscillate between 0.3 and 1.0 opacity for a smooth pulsing effect
                let opacity = 0.3 + 0.7 * (phase.sin() * 0.5 + 0.5);

                material.color.set_alpha(opacity);
            } else {
                // Not invincible - ensure full opacity
                material.color.set_alpha(1.0);
            }
        }
    }
}
