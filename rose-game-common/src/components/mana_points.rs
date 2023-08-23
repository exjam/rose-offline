use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect)]
pub struct ManaPoints {
    pub mp: i32,
}

impl ManaPoints {
    pub fn new(mp: i32) -> Self {
        Self { mp }
    }
}
