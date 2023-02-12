use bevy::ecs::prelude::{Component, Entity};

#[derive(Component, Clone)]
pub struct Owner {
    pub entity: Entity,
}

impl Owner {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
