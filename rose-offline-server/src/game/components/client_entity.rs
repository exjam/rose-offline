use bevy::ecs::prelude::Component;

use rose_data::ZoneId;

pub use rose_game_common::messages::ClientEntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientEntityType {
    Character,
    Monster,
    Npc,
    ItemDrop,
}

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
