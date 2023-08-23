use bevy::{ecs::prelude::Entity, prelude::Event};

use rose_data::QuestTriggerHash;

#[derive(Event)]
pub struct QuestTriggerEvent {
    pub trigger_entity: Entity,
    pub trigger_hash: QuestTriggerHash,
}
