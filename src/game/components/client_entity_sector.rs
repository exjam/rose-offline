use bevy_ecs::prelude::Component;
use bevy_math::UVec2;

#[derive(Component, Clone, Debug)]
pub struct ClientEntitySector {
    pub sector: UVec2,
}

impl ClientEntitySector {
    pub fn new(sector: UVec2) -> Self {
        Self { sector }
    }
}
