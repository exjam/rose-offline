use rose_file_readers::VfsPathBuf;
use std::collections::{hash_map::Iter, HashMap};

use crate::ZoneId;

pub struct ZoneListEntry {
    pub id: ZoneId,
    pub name: String,
    pub zon_file_path: VfsPathBuf,
    pub zsc_cnst_path: VfsPathBuf,
    pub zsc_deco_path: VfsPathBuf,
}

pub struct ZoneList {
    pub zones: HashMap<ZoneId, ZoneListEntry>,
}

impl ZoneList {
    pub fn new(zones: HashMap<ZoneId, ZoneListEntry>) -> Self {
        Self { zones }
    }

    pub fn iter(&self) -> Iter<'_, ZoneId, ZoneListEntry> {
        self.zones.iter()
    }

    pub fn get_zone(&self, id: ZoneId) -> Option<&ZoneListEntry> {
        self.zones.get(&id)
    }
}
