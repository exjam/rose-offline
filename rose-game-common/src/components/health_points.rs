use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct HealthPoints {
    pub hp: i32,
}

impl HealthPoints {
    pub fn new(hp: i32) -> Self {
        Self { hp }
    }
}
