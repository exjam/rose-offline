use bevy::{
    ecs::prelude::Component,
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect, FromReflect)]
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
