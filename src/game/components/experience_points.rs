use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct ExperiencePoints {
    pub xp: u64,
}

impl ExperiencePoints {
    pub fn new() -> Self {
        Self { xp: 0 }
    }
}
