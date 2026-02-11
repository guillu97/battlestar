use serde::{Deserialize, Serialize};
use crate::{entities::Ship, Vec2};

/// Delta update containing only changed entities
///
/// Instead of sending the entire game state every tick (~500 bytes for 5 players),
/// we only send what changed (~50 bytes), achieving 90% bandwidth reduction.
///
/// Full state is still sent periodically (every 100 ticks = 5 seconds) to:
/// - Handle packet loss
/// - Sync new clients
/// - Prevent drift from accumulating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaState {
    /// Game tick number
    pub tick: u64,

    /// Ships that changed this tick (moved, rotated, respawned)
    pub changed_ships: Vec<ShipUpdate>,

    /// Ships that were removed this tick (player disconnected)
    pub removed_ship_ids: Vec<u32>,

    /// Whether this is a full state update (every N ticks)
    pub is_full_state: bool,
}

/// Compressed ship update containing only essential fields
///
/// Saves bandwidth by:
/// - Using f32 instead of f64
/// - Omitting unchanged fields (color rarely changes)
/// - Position and velocity are the minimum needed for interpolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipUpdate {
    pub id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32,

    /// Color only included when ship spawns/respawns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<crate::entities::Color>,
}

impl ShipUpdate {
    /// Create update from ship (include color for spawns)
    pub fn from_ship(ship: &Ship, include_color: bool) -> Self {
        Self {
            id: ship.id,
            position: ship.position,
            velocity: ship.velocity,
            rotation: ship.rotation,
            color: if include_color {
                Some(ship.color)
            } else {
                None
            },
        }
    }

    /// Create update with color (for spawns/respawns)
    pub fn with_color(ship: &Ship) -> Self {
        Self::from_ship(ship, true)
    }

    /// Create update without color (for regular movement)
    pub fn without_color(ship: &Ship) -> Self {
        Self::from_ship(ship, false)
    }
}

impl DeltaState {
    /// Create empty delta state
    pub fn new(tick: u64, is_full_state: bool) -> Self {
        Self {
            tick,
            changed_ships: Vec::new(),
            removed_ship_ids: Vec::new(),
            is_full_state,
        }
    }

    /// Add a ship update to this delta
    pub fn add_ship_update(&mut self, update: ShipUpdate) {
        self.changed_ships.push(update);
    }

    /// Add a removed ship ID
    pub fn add_removed_ship(&mut self, id: u32) {
        self.removed_ship_ids.push(id);
    }

    /// Check if delta contains any changes
    pub fn has_changes(&self) -> bool {
        !self.changed_ships.is_empty() || !self.removed_ship_ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Color;

    fn create_test_ship(id: u32) -> Ship {
        Ship::new(id, Color::RED)
    }

    #[test]
    fn test_delta_state_creation() {
        let delta = DeltaState::new(42, false);
        assert_eq!(delta.tick, 42);
        assert!(!delta.is_full_state);
        assert!(!delta.has_changes());
    }

    #[test]
    fn test_ship_update_with_color() {
        let ship = create_test_ship(1);
        let update = ShipUpdate::with_color(&ship);

        assert_eq!(update.id, 1);
        assert!(update.color.is_some());
        assert_eq!(update.color.unwrap(), Color::RED);
    }

    #[test]
    fn test_ship_update_without_color() {
        let ship = create_test_ship(1);
        let update = ShipUpdate::without_color(&ship);

        assert_eq!(update.id, 1);
        assert!(update.color.is_none());
    }

    #[test]
    fn test_delta_add_ship_update() {
        let mut delta = DeltaState::new(1, false);
        let ship = create_test_ship(1);

        delta.add_ship_update(ShipUpdate::with_color(&ship));

        assert!(delta.has_changes());
        assert_eq!(delta.changed_ships.len(), 1);
    }

    #[test]
    fn test_delta_add_removed_ship() {
        let mut delta = DeltaState::new(1, false);
        delta.add_removed_ship(42);

        assert!(delta.has_changes());
        assert_eq!(delta.removed_ship_ids.len(), 1);
        assert_eq!(delta.removed_ship_ids[0], 42);
    }
}
