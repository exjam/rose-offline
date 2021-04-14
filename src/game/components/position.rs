use serde::{Deserialize, Serialize};
use std::f32;

#[derive(Clone, Deserialize, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: u16,
    pub zone: u16,
    pub respawn_zone: u16,
}

impl Position {
    pub fn distance(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z as f32 - other.z as f32;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
