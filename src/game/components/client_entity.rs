use bevy_ecs::prelude::Component;
use nalgebra::Point2;

use crate::data::ZoneId;

#[derive(Clone, Debug)]
pub enum ClientEntityType {
    Character,
    Monster,
    Npc,
    ItemDrop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientEntityId(pub usize);

#[derive(Component, Clone, Debug)]
pub struct ClientEntity {
    pub id: ClientEntityId,
    pub zone_id: ZoneId,
    pub sector: Point2<u32>,
    pub entity_type: ClientEntityType,
}

impl ClientEntity {
    pub fn new(
        entity_type: ClientEntityType,
        id: ClientEntityId,
        zone_id: ZoneId,
        sector: Point2<u32>,
    ) -> Self {
        Self {
            id,
            zone_id,
            sector,
            entity_type,
        }
    }
}
