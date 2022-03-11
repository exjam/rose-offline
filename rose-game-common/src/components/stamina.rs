use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

pub const MAX_STAMINA: u32 = 5000;

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Stamina {
    pub stamina: u32,
}

impl Stamina {
    pub fn new() -> Self {
        Self { stamina: 0 }
    }
}

impl Default for Stamina {
    fn default() -> Self {
        Self::new()
    }
}
