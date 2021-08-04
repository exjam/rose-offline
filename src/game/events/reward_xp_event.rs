use bevy_ecs::prelude::Entity;

pub struct RewardXpEvent {
    pub entity: Entity,
    pub xp: u64,
    pub stamina: u32,
    pub source: Option<Entity>,
}

impl RewardXpEvent {
    pub fn new(entity: Entity, xp: u64, stamina: u32, source: Option<Entity>) -> Self {
        Self {
            entity,
            xp,
            stamina,
            source,
        }
    }
}
