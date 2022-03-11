use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use rose_data::ZoneId;

#[derive(Clone, Debug)]
pub enum ClientEntityType {
    Character,
    Monster,
    Npc,
    ItemDrop,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientEntityId(pub usize);

#[derive(Component, Clone, Debug)]
pub struct ClientEntity {
    pub id: ClientEntityId,
    pub zone_id: ZoneId,
    pub entity_type: ClientEntityType,
}

impl ClientEntity {
    pub fn new(entity_type: ClientEntityType, id: ClientEntityId, zone_id: ZoneId) -> Self {
        Self {
            id,
            zone_id,
            entity_type,
        }
    }
}
