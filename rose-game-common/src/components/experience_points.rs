use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct ExperiencePoints {
    pub xp: u64,
}

impl ExperiencePoints {
    pub fn new(xp: u64) -> Self {
        Self { xp }
    }
}

impl Default for ExperiencePoints {
    fn default() -> Self {
        Self::new(0)
    }
}
