use bevy::ecs::prelude::Component;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct Destination {
    pub position: Vec3,
}

impl Destination {
    pub fn new(position: Vec3) -> Self {
        Self { position }
    }
}
