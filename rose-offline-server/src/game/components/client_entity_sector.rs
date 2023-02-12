use bevy::ecs::prelude::Component;
use bevy::math::UVec2;

#[derive(Component, Clone, Debug)]
pub struct ClientEntitySector {
    pub sector: UVec2,
}

impl ClientEntitySector {
    pub fn new(sector: UVec2) -> Self {
        Self { sector }
    }
}
