use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect)]
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
