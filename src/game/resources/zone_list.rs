use std::collections::HashMap;

use bevy_ecs::prelude::Entity;

use crate::data::{NpcId, ZoneId};

struct ZoneData {
    local_npcs: HashMap<NpcId, Entity>,
}

pub struct ZoneList {
    zones: HashMap<ZoneId, ZoneData>,
}

impl ZoneList {
    pub fn new() -> Self {
        Self {
            zones: Default::default(),
        }
    }

    pub fn add_zone(&mut self, zone_id: ZoneId) {
        self.zones.insert(
            zone_id,
            ZoneData {
                local_npcs: Default::default(),
            },
        );
    }

    pub fn add_local_npc(&mut self, zone_id: ZoneId, npc_id: NpcId, entity: Entity) {
        if let Some(zone) = self.zones.get_mut(&zone_id) {
            zone.local_npcs.insert(npc_id, entity);
        }
    }

    pub fn find_local_npc(&self, zone_id: ZoneId, npc_id: NpcId) -> Option<Entity> {
        self.zones
            .get(&zone_id)
            .and_then(|zone| zone.local_npcs.get(&npc_id).cloned())
    }
}
