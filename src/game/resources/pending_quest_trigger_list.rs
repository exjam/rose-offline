use crate::data::QuestTriggerHash;
use legion::Entity;

pub struct PendingQuestTrigger {
    pub trigger_entity: Entity,
    pub trigger_hash: QuestTriggerHash,
}

pub type PendingQuestTriggerList = Vec<PendingQuestTrigger>;
