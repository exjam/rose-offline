use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct StatPoints {
    pub points: u32,
}

impl StatPoints {
    pub fn new(points: u32) -> Self {
        Self { points }
    }
}

impl Default for StatPoints {
    fn default() -> Self {
        Self::new(0)
    }
}
