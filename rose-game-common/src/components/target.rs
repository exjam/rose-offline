use bevy::ecs::prelude::{Component, Entity};

#[derive(Component, Clone)]
pub struct Target {
    pub entity: Entity,
}

impl Target {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
