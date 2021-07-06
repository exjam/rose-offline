use nalgebra::Point3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Destination {
    pub position: Point3<f32>,
}

impl Destination {
    pub fn new(position: Point3<f32>) -> Self {
        Self { position }
    }
}
