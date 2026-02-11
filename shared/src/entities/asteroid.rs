use serde::{Deserialize, Serialize};
use crate::math::Vec2;
use crate::physics::wrap_position;

/// Asteroid entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asteroid {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub radius: f32,
}

impl Asteroid {
    pub fn new(id: u32, position: Vec2, velocity: Vec2, radius: f32) -> Self {
        Self {
            id,
            position,
            velocity,
            radius,
        }
    }

    /// Update asteroid position (simple velocity integration with wrapping)
    pub fn update(&mut self, dt: f32, world_limit: f32) {
        self.position.x += self.velocity.x * dt;
        self.position.y += self.velocity.y * dt;
        wrap_position(&mut self.position, world_limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asteroid_creation() {
        let asteroid = Asteroid::new(1, Vec2::new(100.0, 100.0), Vec2::new(10.0, 10.0), 20.0);
        assert_eq!(asteroid.id, 1);
        assert_eq!(asteroid.position, Vec2::new(100.0, 100.0));
        assert_eq!(asteroid.radius, 20.0);
    }

    #[test]
    fn test_asteroid_update() {
        let mut asteroid = Asteroid::new(1, Vec2::ZERO, Vec2::new(100.0, 50.0), 20.0);
        asteroid.update(0.1, 2000.0);

        assert_eq!(asteroid.position.x, 10.0);
        assert_eq!(asteroid.position.y, 5.0);
    }

    #[test]
    fn test_asteroid_wrapping() {
        let mut asteroid = Asteroid::new(1, Vec2::new(1999.0, 0.0), Vec2::new(100.0, 0.0), 20.0);
        asteroid.update(0.1, 2000.0);

        // Should wrap to the other side
        assert!(asteroid.position.x < 0.0, "Asteroid should wrap to negative side");
    }
}
