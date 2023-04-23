use bevy::{
    ecs::prelude::Component,
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize, Reflect, FromReflect)]
pub struct MoveSpeed {
    pub speed: f32,
}

impl MoveSpeed {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}
