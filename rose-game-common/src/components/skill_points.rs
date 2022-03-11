use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SkillPoints {
    pub points: u32,
}

impl SkillPoints {
    pub fn new() -> Self {
        Self { points: 0 }
    }
}

impl Default for SkillPoints {
    fn default() -> Self {
        Self::new()
    }
}
