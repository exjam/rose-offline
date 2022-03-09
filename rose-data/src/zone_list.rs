use rose_file_readers::VfsPathBuf;

use crate::ZoneId;

pub struct ZoneListEntry {
    pub id: ZoneId,
    pub name: String,
    pub zon_file_path: VfsPathBuf,
    pub zsc_cnst_path: VfsPathBuf,
    pub zsc_deco_path: VfsPathBuf,
}

pub struct ZoneList {
    zones: Vec<Option<ZoneListEntry>>,
}

impl ZoneList {
    pub fn new(zones: Vec<Option<ZoneListEntry>>) -> Self {
        Self { zones }
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
