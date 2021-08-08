use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Iter, HashMap},
    num::NonZeroU16,
    str::FromStr,
};

use nalgebra::{distance, Point2, Point3};

use super::npc_database::{NpcConversationId, NpcId};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct ZoneId(pub NonZeroU16);

id_wrapper_impl!(ZoneId, NonZeroU16, u16);

pub struct ZoneMonsterSpawnPoint {
    pub position: Point3<f32>,
    pub basic_spawns: Vec<(NpcId, usize)>,
    pub tactic_spawns: Vec<(NpcId, usize)>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
}

pub struct ZoneNpcSpawn {
    pub npc_id: NpcId,
    pub position: Point3<f32>,
    pub direction: f32,
    pub conversation: NpcConversationId,
}

pub struct ZoneEventObject {
    pub event_id: u16,
    pub map_chunk_x: i32,
    pub map_chunk_y: i32,
    pub position: Point3<f32>,
}

pub struct ZoneData {
    pub id: ZoneId,
    pub name: String,
    pub sector_size: u32,
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub event_objects: Vec<ZoneEventObject>,
    pub monster_spawns: Vec<ZoneMonsterSpawnPoint>,
    pub npcs: Vec<ZoneNpcSpawn>,
    pub sectors_base_position: Point2<f32>,
    pub num_sectors_x: u32,
    pub num_sectors_y: u32,
    pub start_position: Point3<f32>,
    pub revive_positions: Vec<Point3<f32>>,
    pub day_cycle: u32,
    pub morning_time: u32,
    pub day_time: u32,
    pub evening_time: u32,
    pub night_time: u32,
}

impl ZoneData {
    pub fn get_closest_revive_position(&self, origin: Point3<f32>) -> Option<Point3<f32>> {
        let mut closest = None;

        for revive_position in self.revive_positions.iter() {
            let distance = distance(&revive_position.xy(), &origin.xy());

            if closest.map_or(true, |(d, _)| distance < d) {
                closest = Some((distance, revive_position));
            }
        }

        closest.map(|(_, p)| *p)
    }
}

pub struct ZoneDatabase {
    zones: HashMap<ZoneId, ZoneData>,
}

impl ZoneDatabase {
    pub fn new(zones: HashMap<ZoneId, ZoneData>) -> Self {
        Self { zones }
    }

    pub fn iter(&self) -> Iter<'_, ZoneId, ZoneData> {
        self.zones.iter()
    }

    pub fn get_zone(&self, id: ZoneId) -> Option<&ZoneData> {
        self.zones.get(&id)
    }
}
