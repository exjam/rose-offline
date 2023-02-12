use bevy::ecs::prelude::Component;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

use rose_data::ZoneId;

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct Position {
    pub position: Vec3,
    pub zone_id: ZoneId,
}

impl Position {
    pub fn new(position: Vec3, zone_id: ZoneId) -> Self {
        Self { position, zone_id }
    }
}
