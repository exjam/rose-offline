use bevy::{
    math::{Vec2, Vec3, Vec3Swizzles},
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num::NonZeroU16, str::FromStr, sync::Arc};

use crate::{NpcConversationId, NpcId, SkyboxId, StringDatabase};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Reflect, FromReflect)]
pub struct ZoneId(pub NonZeroU16);

id_wrapper_impl!(ZoneId, NonZeroU16, u16);

pub struct ZoneMonsterSpawnPoint {
    pub position: Vec3,
    pub basic_spawns: Vec<(NpcId, usize)>,
    pub tactic_spawns: Vec<(NpcId, usize)>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
}

pub struct ZoneNpcSpawn {
    pub npc_id: NpcId,
    pub position: Vec3,
    pub direction: f32,
    pub conversation: NpcConversationId,
}

pub struct ZoneEventObject {
    pub event_id: u16,
    pub map_chunk_x: i32,
    pub map_chunk_y: i32,
    pub position: Vec3,
}

pub struct ZoneData {
    pub id: ZoneId,
    pub name: &'static str,
    pub description: &'static str,
    pub sector_size: u32,
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub event_objects: Vec<ZoneEventObject>,
    pub monster_spawns: Vec<ZoneMonsterSpawnPoint>,
    pub npcs: Vec<ZoneNpcSpawn>,
    pub sectors_base_position: Vec2,
    pub num_sectors_x: u32,
    pub num_sectors_y: u32,
    pub start_position: Vec3,
    pub revive_positions: Vec<Vec3>,
    pub event_positions: HashMap<String, Vec3>,
    pub day_cycle: u32,
    pub morning_time: u32,
    pub day_time: u32,
    pub evening_time: u32,
    pub night_time: u32,
    pub skybox_id: Option<SkyboxId>,
}

impl ZoneData {
    pub fn get_closest_revive_position(&self, origin: Vec3) -> Option<Vec3> {
        let mut closest = None;

        for revive_position in self.revive_positions.iter() {
            let distance = revive_position.xy().distance(origin.xy());

            if closest.map_or(true, |(d, _)| distance < d) {
                closest = Some((distance, revive_position));
            }
        }

        closest.map(|(_, p)| *p)
    }
}

pub struct ZoneDatabase {
    _string_database: Arc<StringDatabase>,
    zones: Vec<Option<ZoneData>>,
}

impl ZoneDatabase {
    pub fn new(string_database: Arc<StringDatabase>, zones: Vec<Option<ZoneData>>) -> Self {
        Self {
            _string_database: string_database,
            zones,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ZoneData> {
        self.zones.iter().filter_map(|zone_data| zone_data.as_ref())
    }

    pub fn get_zone(&self, id: ZoneId) -> Option<&ZoneData> {
        match self.zones.get(id.get() as usize) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }
}
