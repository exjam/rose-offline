use bevy::ecs::prelude::Entity;

use rose_data::QuestTriggerHash;

pub struct QuestTriggerEvent {
    pub trigger_entity: Entity,
    pub trigger_hash: QuestTriggerHash,
}
