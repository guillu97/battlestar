use crate::math::Vec2;
use super::PhysicsConstants;

/// Input for ship control
#[derive(Debug, Clone, Copy)]
pub struct Input {
    pub thrust: f32,
    pub rotate: f32,
}

impl Input {
    pub fn new(thrust: f32, rotate: f32) -> Self {
        Self { thrust, rotate }
    }

    pub fn clamp(&mut self) {
        self.thrust = self.thrust.clamp(-1.0, 1.0);
        self.rotate = self.rotate.clamp(-1.0, 1.0);
    }
}

/// Apply ship physics (shared between client and server)
/// This is the single source of truth for ship movement
pub fn apply_ship_physics(
    position: &mut Vec2,
    velocity: &mut Vec2,
    rotation: &mut f32,
    input: &Input,
    dt: f32,
    constants: &PhysicsConstants,
) {
    // Apply rotation - NEGATIVE because D key should rotate clockwise
    *rotation -= input.rotate * constants.rotation_speed * dt;

    // Apply thrust in facing direction
    // At rotation=0, ship points UP (Y+), not RIGHT
    // So we need: x = -sin(rotation), y = cos(rotation)
    velocity.x -= input.thrust * rotation.sin() * constants.thrust_accel * dt;
    velocity.y += input.thrust * rotation.cos() * constants.thrust_accel * dt;

    // Apply drag (friction)
    let drag_factor = constants.drag.powf(dt * 60.0);
    velocity.x *= drag_factor;
    velocity.y *= drag_factor;

    // Clamp to max speed
    let speed = velocity.length();
    if speed > constants.max_speed {
        let scale = constants.max_speed / speed;
        velocity.x *= scale;
        velocity.y *= scale;
    }

    // Apply velocity (position integration)
    position.x += velocity.x * dt;
    position.y += velocity.y * dt;

    // Wrap position at world boundaries
    wrap_position(position, constants.world_limit);
}

/// Wrap position when reaching world boundaries (toroidal world)
pub fn wrap_position(pos: &mut Vec2, world_limit: f32) {
    if pos.x > world_limit {
        pos.x = -world_limit;
    } else if pos.x < -world_limit {
        pos.x = world_limit;
    }

    if pos.y > world_limit {
        pos.y = -world_limit;
    } else if pos.y < -world_limit {
        pos.y = world_limit;
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
    fn test_thrust_increases_velocity() {
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::ZERO;
        let mut rotation = 0.0;
        let input = Input::new(1.0, 0.0);
        let constants = test_constants();

        apply_ship_physics(&mut pos, &mut vel, &mut rotation, &input, 0.016, &constants);

        // Ship points up (Y+) at rotation=0, so thrust should increase Y velocity
        assert!(vel.y > 0.0, "Y velocity should increase with forward thrust");
        assert_eq!(vel.x, 0.0, "X velocity should remain zero");
    }

    #[test]
    fn test_rotation_changes_angle() {
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::ZERO;
        let mut rotation = 0.0;
        let input = Input::new(0.0, 1.0);
        let constants = test_constants();

        apply_ship_physics(&mut pos, &mut vel, &mut rotation, &input, 0.016, &constants);

        // Rotation should decrease (clockwise) with positive rotate input
        assert!(rotation < 0.0, "Rotation should be negative (clockwise)");
    }

    #[test]
    fn test_drag_reduces_velocity() {
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::new(100.0, 100.0);
        let mut rotation = 0.0;
        let input = Input::new(0.0, 0.0);
        let constants = test_constants();

        let initial_speed = vel.length();

        apply_ship_physics(&mut pos, &mut vel, &mut rotation, &input, 0.016, &constants);

        let final_speed = vel.length();
        assert!(final_speed < initial_speed, "Drag should reduce velocity");
    }

    #[test]
    fn test_max_speed_clamping() {
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::ZERO;
        let mut rotation = 0.0;
        let input = Input::new(1.0, 0.0);
        let constants = test_constants();

        // Apply thrust for many frames to reach max speed
        for _ in 0..1000 {
            apply_ship_physics(&mut pos, &mut vel, &mut rotation, &input, 0.016, &constants);
        }

        let speed = vel.length();
        assert!(
            speed <= constants.max_speed + 0.1,
            "Speed should not exceed max_speed (got {}, max {})",
            speed,
            constants.max_speed
        );
    }

    #[test]
    fn test_wrap_position_right_edge() {
        let mut pos = Vec2::new(2001.0, 0.0);
        wrap_position(&mut pos, 2000.0);
        assert_eq!(pos.x, -2000.0);
    }

    #[test]
    fn test_wrap_position_left_edge() {
        let mut pos = Vec2::new(-2001.0, 0.0);
        wrap_position(&mut pos, 2000.0);
        assert_eq!(pos.x, 2000.0);
    }

    #[test]
    fn test_wrap_position_top_edge() {
        let mut pos = Vec2::new(0.0, 2001.0);
        wrap_position(&mut pos, 2000.0);
        assert_eq!(pos.y, -2000.0);
    }

    #[test]
    fn test_wrap_position_bottom_edge() {
        let mut pos = Vec2::new(0.0, -2001.0);
        wrap_position(&mut pos, 2000.0);
        assert_eq!(pos.y, 2000.0);
    }

    #[test]
    fn test_position_integration() {
        let mut pos = Vec2::ZERO;
        let mut vel = Vec2::new(100.0, 50.0);
        let mut rotation = 0.0;
        let input = Input::new(0.0, 0.0);
        let constants = test_constants();

        apply_ship_physics(&mut pos, &mut vel, &mut rotation, &input, 0.1, &constants);

        // Position should increase by velocity * dt (accounting for drag)
        assert!(pos.x > 0.0, "X position should increase");
        assert!(pos.y > 0.0, "Y position should increase");
    }
}
