use bevy::ecs::prelude::{Component, Entity};

#[derive(Component, Clone)]
pub struct PartyOwner {
    pub entity: Entity,
}

impl PartyOwner {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
