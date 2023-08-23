use bevy::{ecs::prelude::Entity, prelude::Event};

#[derive(Event)]
pub struct RewardXpEvent {
    pub entity: Entity,
    pub xp: u64,
    pub stamina: bool,
    pub source: Option<Entity>,
}

impl RewardXpEvent {
    pub fn new(entity: Entity, xp: u64, stamina: bool, source: Option<Entity>) -> Self {
        Self {
            entity,
            xp,
            stamina,
            source,
        }
    }
}
