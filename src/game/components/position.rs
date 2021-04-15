use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::f32;

#[derive(Clone, Deserialize, Serialize)]
pub struct Position {
    pub position: Vector3<f32>,
    pub zone: u16,
    pub respawn_zone: u16,
}
