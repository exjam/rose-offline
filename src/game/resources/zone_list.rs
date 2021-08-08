use std::collections::HashMap;

use bevy_ecs::prelude::Entity;

use crate::data::{NpcId, ZoneId};

#[derive(Hash, PartialEq, Eq)]
struct EventObjectKey {
    event_id: u16,
    map_chunk_x: i32,
    map_chunk_y: i32,
}

struct ZoneData {
    event_objects: HashMap<EventObjectKey, Entity>,
}

pub struct ZoneList {
    zones: HashMap<ZoneId, ZoneData>,
    npcs: HashMap<NpcId, Entity>,
}

impl ZoneList {
    pub fn new() -> Self {
        Self {
            zones: Default::default(),
            npcs: Default::default(),
        }
    }

    pub fn add_zone(&mut self, zone_id: ZoneId) {
        self.zones.insert(
            zone_id,
            ZoneData {
                event_objects: Default::default(),
            },
        );
    }

    pub fn add_event_object(
        &mut self,
        zone_id: ZoneId,
        event_id: u16,
        map_chunk_x: i32,
        map_chunk_y: i32,
        entity: Entity,
    ) {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.event_objects.insert(
                EventObjectKey {
                    event_id,
                    map_chunk_x,
                    map_chunk_y,
                },
                entity,
            );
        }
    }

    pub fn find_event_object(
        &self,
        zone_id: ZoneId,
        event_id: u16,
        map_chunk_x: i32,
        map_chunk_y: i32,
    ) -> Option<Entity> {
        self.zones.get(&zone_id).and_then(|zone| {
            zone.event_objects
                .get(&EventObjectKey {
                    event_id,
                    map_chunk_x,
                    map_chunk_y,
                })
                .cloned()
        })
    }

    pub fn add_npc(&mut self, npc_id: NpcId, entity: Entity) {
        self.npcs.insert(npc_id, entity);
    }

    pub fn find_npc(&self, npc_id: NpcId) -> Option<Entity> {
        self.npcs.get(&npc_id).cloned()
    }
}
