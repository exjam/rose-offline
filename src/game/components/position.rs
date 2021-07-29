use nalgebra::Point3;
use serde::{Deserialize, Serialize};

use crate::data::ZoneId;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Point3<f32>,
    pub zone_id: ZoneId,
}

impl Position {
    pub fn new(position: Point3<f32>, zone_id: ZoneId) -> Self {
        Self { position, zone_id }
    }
}
