use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Level {
    pub level: u32,
}

impl Default for Level {
    fn default() -> Self {
        Self { level: 1 }
    }
}

impl Level {
    pub fn new(level: u32) -> Self {
        Self { level }
    }
}
