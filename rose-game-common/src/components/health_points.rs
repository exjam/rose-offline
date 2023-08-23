use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct HealthPoints {
    pub hp: i32,
}

impl HealthPoints {
    pub fn new(hp: i32) -> Self {
        Self { hp }
    }
}
