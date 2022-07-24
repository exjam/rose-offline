use rose_file_readers::VfsPathBuf;

use crate::{SkyboxId, ZoneId};

pub struct ZoneListEntry {
    pub id: ZoneId,
    pub name: String,
    pub minimap_path: Option<VfsPathBuf>,
    pub minimap_start_x: u32,
    pub minimap_start_y: u32,
    pub zon_file_path: VfsPathBuf,
    pub zsc_cnst_path: VfsPathBuf,
    pub zsc_deco_path: VfsPathBuf,
    pub day_cycle: u32,
    pub morning_time: u32,
    pub day_time: u32,
    pub evening_time: u32,
    pub night_time: u32,
    pub skybox_id: Option<SkyboxId>,
    pub background_music_day: Option<VfsPathBuf>,
    pub background_music_night: Option<VfsPathBuf>,
    pub footstep_type: Option<u32>,
}

pub struct ZoneList {
    zones: Vec<Option<ZoneListEntry>>,
}

impl ZoneList {
    pub fn new(zones: Vec<Option<ZoneListEntry>>) -> Self {
        Self { zones }
    }

    pub fn len(&self) -> usize {
        self.zones.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ZoneListEntry> {
        self.zones
            .iter()
            .filter_map(|zone_list_entry| zone_list_entry.as_ref())
    }

    pub fn get_zone(&self, id: ZoneId) -> Option<&ZoneListEntry> {
        match self.zones.get(id.get() as usize) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }
}
