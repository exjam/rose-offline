use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: u16,
    pub zone: u16,
    pub respawn_zone: u16,
}
