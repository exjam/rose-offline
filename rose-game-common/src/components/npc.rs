use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use rose_data::NpcId;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Npc {
    pub id: NpcId,
    pub quest_index: u16,
}

impl Npc {
    pub fn new(id: NpcId, quest_index: u16) -> Self {
        Self { id, quest_index }
    }
}
