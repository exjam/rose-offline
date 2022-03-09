use bevy_ecs::prelude::Entity;
use std::collections::HashMap;

use rose_data::{NpcId, ZoneId};

#[derive(Hash, PartialEq, Eq)]
struct EventObjectKey {
    event_id: u16,
    map_chunk_x: i32,
    map_chunk_y: i32,
}

struct ZoneData {
    monster_spawns_enabled: bool,
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
                monster_spawns_enabled: true,
                event_objects: Default::default(),
            },
        );
    }

    pub fn get_monster_spawns_enabled(&self, zone_id: ZoneId) -> bool {
        self.zones
            .get(&zone_id)
            .map(|zone| zone.monster_spawns_enabled)
            .unwrap_or(false)
    }

    pub fn set_monster_spawns_enabled(&mut self, zone_id: ZoneId, enabled: bool) -> bool {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.monster_spawns_enabled = enabled;
            true
        } else {
            false
        }
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
