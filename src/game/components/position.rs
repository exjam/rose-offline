use nalgebra::Point3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Point3<f32>,
    pub zone: u16,
}

impl Position {
    pub fn new(position: Point3<f32>, zone: u16) -> Self {
        Self { position, zone }
    }
}
