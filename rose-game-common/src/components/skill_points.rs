use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SkillPoints {
    pub points: u32,
}

impl SkillPoints {
    pub fn new(points: u32) -> Self {
        Self { points }
    }
}

impl Default for SkillPoints {
    fn default() -> Self {
        Self::new(0)
    }
}
