use bevy_ecs::prelude::Component;
use nalgebra::Point2;

#[derive(Component, Clone, Debug)]
pub struct ClientEntitySector {
    pub sector: Point2<u32>,
}

impl ClientEntitySector {
    pub fn new(sector: Point2<u32>) -> Self {
        Self { sector }
    }
}
