use bevy_ecs::prelude::Entity;

#[derive(Clone)]
pub struct Owner {
    pub entity: Entity,
}

impl Owner {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
