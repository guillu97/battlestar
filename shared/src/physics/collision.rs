use crate::math::Vec2;

/// Calculate distance between two points
pub fn distance(p1: Vec2, p2: Vec2) -> f32 {
    p1.distance_to(p2)
}

/// Check collision between two circular objects
pub fn check_collision(pos1: Vec2, radius1: f32, pos2: Vec2, radius2: f32) -> bool {
    let collision_distance = radius1 + radius2;
    distance(pos1, pos2) < collision_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let p1 = Vec2::new(0.0, 0.0);
        let p2 = Vec2::new(3.0, 4.0);
        assert_eq!(distance(p1, p2), 5.0);
    }

    #[test]
    fn test_collision_detection_overlapping() {
        let pos1 = Vec2::new(0.0, 0.0);
        let pos2 = Vec2::new(10.0, 0.0);

        assert!(check_collision(pos1, 15.0, pos2, 5.0), "Should collide when circles overlap");
    }

    #[test]
    fn test_collision_detection_not_overlapping() {
        let pos1 = Vec2::new(0.0, 0.0);
        let pos2 = Vec2::new(100.0, 0.0);

        assert!(!check_collision(pos1, 10.0, pos2, 10.0), "Should not collide when far apart");
    }

    #[test]
    fn test_collision_detection_touching() {
        let pos1 = Vec2::new(0.0, 0.0);
        let pos2 = Vec2::new(20.0, 0.0);

        // Exactly touching (distance == sum of radii) should NOT collide (< not <=)
        assert!(!check_collision(pos1, 10.0, pos2, 10.0), "Should not collide when exactly touching");
    }
}
