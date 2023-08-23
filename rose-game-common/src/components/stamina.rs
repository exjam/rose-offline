use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

pub const MAX_STAMINA: u32 = 5000;

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct Stamina {
    pub stamina: u32,
}

impl Stamina {
    pub fn new(stamina: u32) -> Self {
        Self { stamina }
    }
}

impl Default for Stamina {
    fn default() -> Self {
        Self::new(0)
    }
}
