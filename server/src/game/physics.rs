use super::types::Vec2;
use crate::constants::WORLD_LIMIT;

pub fn wrap_position(pos: &mut Vec2) {
    let limit = WORLD_LIMIT;
    if pos.x > limit {
        pos.x = -limit;
    } else if pos.x < -limit {
        pos.x = limit;
    }

    if pos.y > limit {
        pos.y = -limit;
    } else if pos.y < -limit {
        pos.y = limit;
    }
}

pub fn distance(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}
