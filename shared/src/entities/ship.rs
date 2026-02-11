use serde::{Deserialize, Serialize};
use crate::math::Vec2;
use crate::physics::{PhysicsConstants, Input, apply_ship_physics, wrap_position};
use super::Color;

/// Ship entity with behavior methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,
    pub color: Color,
}

impl Ship {
    pub fn new(id: u32, color: Color) -> Self {
        Self {
            id,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            rotation: 0.0,
            color,
        }
    }

    /// Apply player input to the ship
    pub fn apply_input(&mut self, input: &Input, dt: f32, constants: &PhysicsConstants) {
        apply_ship_physics(
            &mut self.position,
            &mut self.velocity,
            &mut self.rotation,
            input,
            dt,
            constants,
        );
    }

    /// Update ship physics without input (drag, position integration, wrapping)
    pub fn update(&mut self, dt: f32, constants: &PhysicsConstants) {
        // Apply drag
        let drag_factor = constants.drag.powf(dt * 60.0);
        self.velocity *= drag_factor;

        // Clamp to max speed
        let speed = self.velocity.length();
        if speed > constants.max_speed {
            let scale = constants.max_speed / speed;
            self.velocity *= scale;
        }

        // Apply velocity
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;

        // Wrap position
        wrap_position(&mut self.position, constants.world_limit);
    }

    /// Respawn ship at origin (used after collision/death)
    pub fn respawn(&mut self) {
        self.position = Vec2::ZERO;
        self.velocity = Vec2::ZERO;
        self.rotation = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_constants() -> PhysicsConstants {
        PhysicsConstants {
            thrust_accel: 300.0,
            rotation_speed: 4.0,
            max_speed: 400.0,
            drag: 0.95,
            world_limit: 2000.0,
            ship_radius: 25.0,
        }
    }

    #[test]
    fn test_ship_creation() {
        let ship = Ship::new(1, Color::RED);
        assert_eq!(ship.id, 1);
        assert_eq!(ship.position, Vec2::ZERO);
        assert_eq!(ship.velocity, Vec2::ZERO);
        assert_eq!(ship.rotation, 0.0);
    }

    #[test]
    fn test_ship_apply_input() {
        let mut ship = Ship::new(1, Color::RED);
        let input = Input::new(1.0, 0.0);
        let constants = test_constants();

        ship.apply_input(&input, 0.016, &constants);

        assert!(ship.velocity.y > 0.0, "Ship should move forward");
    }

    #[test]
    fn test_ship_update() {
        let mut ship = Ship::new(1, Color::RED);
        ship.velocity = Vec2::new(100.0, 100.0);
        let constants = test_constants();

        let init_speed = ship.velocity.length();
        ship.update(0.016, &constants);
        let final_speed = ship.velocity.length();

        assert!(final_speed < init_speed, "Drag should reduce speed");
    }

    #[test]
    fn test_ship_respawn() {
        let mut ship = Ship::new(1, Color::RED);
        ship.position = Vec2::new(100.0, 100.0);
        ship.velocity = Vec2::new(50.0, 50.0);
        ship.rotation = 1.5;

        ship.respawn();

        assert_eq!(ship.position, Vec2::ZERO);
        assert_eq!(ship.velocity, Vec2::ZERO);
        assert_eq!(ship.rotation, 0.0);
    }
}
