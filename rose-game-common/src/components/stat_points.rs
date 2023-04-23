use bevy::{
    ecs::prelude::Component,
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize, Reflect, FromReflect)]
pub struct StatPoints {
    pub points: u32,
}

impl StatPoints {
    pub fn new(points: u32) -> Self {
        Self { points }
    }
}

impl Default for StatPoints {
    fn default() -> Self {
        Self::new(0)
    }
}
