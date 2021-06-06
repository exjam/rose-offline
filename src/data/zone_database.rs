use std::collections::{hash_map::Iter, HashMap};

use nalgebra::{Point2, Point3};

use super::npc_database::{NpcConversationReference, NpcReference};

pub struct ZoneMonsterSpawnPoint {
    pub position: Point3<f32>,
    pub basic_spawns: Vec<(NpcReference, usize)>,
    pub tactic_spawns: Vec<(NpcReference, usize)>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
}

pub struct ZoneNpcSpawn {
    pub npc: NpcReference,
    pub position: Point3<f32>,
    pub direction: f32,
    pub conversation: NpcConversationReference,
}

pub struct ZoneData {
    pub id: u16,
    pub sector_size: u32,
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub monster_spawns: Vec<ZoneMonsterSpawnPoint>,
    pub npcs: Vec<ZoneNpcSpawn>,
    pub sectors_base_position: Point2<f32>,
    pub num_sectors_x: u32,
    pub num_sectors_y: u32,
}

pub struct ZoneDatabase {
    zones: HashMap<u16, ZoneData>,
}

impl ZoneDatabase {
    pub fn new(zones: HashMap<u16, ZoneData>) -> Self {
        Self { zones }
    }

    pub fn iter(&self) -> Iter<'_, u16, ZoneData> {
        self.zones.iter()
    }

    #[allow(dead_code)]
    pub fn get_zone(&self, id: usize) -> Option<&ZoneData> {
        self.zones.get(&(id as u16))
    }
}
