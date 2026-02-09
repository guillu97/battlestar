// Client-side constants
pub const CAMERA_DECAY_RATE: f32 = 14.0;

// Physics constants (must match server!)
pub const THRUST_ACCEL: f32 = 2000.0;  // pixels/sec²
pub const ROTATION_SPEED: f32 = 6.0;   // radians/sec
pub const MAX_SPEED: f32 = 1000.0;      // pixels/sec
pub const DRAG: f32 = 0.98;            // velocity multiplier per frame
pub const WORLD_LIMIT: f32 = 400.0;    // world boundary for wrapping
