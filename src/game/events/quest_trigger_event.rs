use crate::data::QuestTriggerHash;
use bevy_ecs::prelude::Entity;

pub struct QuestTriggerEvent {
    pub trigger_entity: Entity,
    pub trigger_hash: QuestTriggerHash,
}
