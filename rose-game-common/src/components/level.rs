use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect)]
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
