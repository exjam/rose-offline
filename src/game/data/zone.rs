use nalgebra::Point2;
use std::path::Path;

use super::formats::{
    ifo, zon, FileReader, IfoFile, IfoReadError, VfsIndex, ZonFile, ZonReadError,
};
use super::{STB_ZONE, VFS_INDEX};

const MIN_SECTOR_SIZE: i32 = 5000;
const MAX_SECTOR_SIZE: i32 = 12000;

pub enum ZoneLoadError {
    NotExists,
    ZonFileInvalidPath,
    ZonFileNotFound,
    ZonFileInvalid,
    IfoFileInvalid,
}

impl From<ZonReadError> for ZoneLoadError {
    fn from(_: ZonReadError) -> Self {
        Self::ZonFileInvalid
    }
}

impl From<IfoReadError> for ZoneLoadError {
    fn from(_: IfoReadError) -> Self {
        Self::IfoFileInvalid
    }
}

pub struct ZoneInfo {
    pub id: u16,
    pub sector_size: u32,
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub monster_spawns: Vec<ifo::MonsterSpawnPoint>,
    pub npcs: Vec<ifo::Npc>,
    pub event_objects: Vec<ifo::EventObject>,
    pub event_positions: Vec<zon::EventPosition>,
    pub sectors_base_position: Point2<f32>,
    pub num_sectors_x: u32,
    pub num_sectors_y: u32,
}

pub struct ZoneInfoList {
    pub zones: Vec<Option<ZoneInfo>>,
}

impl ZoneInfoList {
    pub fn load() -> Self {
        let mut zones = Vec::new();
        zones.push(None);
        for i in 1..STB_ZONE.rows() {
            if let Ok(zone) = ZoneInfo::load(i) {
                zones.push(Some(zone));
            } else {
                zones.push(None);
            }
        }
        Self { zones }
    }

    pub fn get_zone_info(&self, id: u16) -> Option<&ZoneInfo> {
        self.zones.get(id as usize).and_then(|x| x.as_ref())
    }
}

impl ZoneInfo {
    pub fn load(index: usize) -> Result<Self, ZoneLoadError> {
        let zone_file = VfsIndex::normalise_path(
            STB_ZONE
                .get_zone_file(index)
                .ok_or(ZoneLoadError::NotExists)?,
        );
        let zone_base_directory = Path::new(&zone_file)
            .parent()
            .ok_or(ZoneLoadError::ZonFileInvalidPath)?;

        let file = VFS_INDEX
            .open_file(&zone_file)
            .ok_or(ZoneLoadError::ZonFileNotFound)?;
        let zon_file = ZonFile::read(FileReader::from(&file))?;

        let mut monster_spawns = Vec::new();
        let mut npcs = Vec::new();
        let mut event_objects = Vec::new();
        let mut ifo_count = 0;

        let mut min_x = 64u32;
        let mut min_y = 64u32;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..64u32 {
            for x in 0..64u32 {
                let ifo_file_path = zone_base_directory.join(format!("{}_{}.IFO", x, y));
                if let Some(file) = VFS_INDEX.open_file(&ifo_file_path.to_string_lossy()) {
                    let ifo_file = IfoFile::read(FileReader::from(&file))?;
                    monster_spawns.extend(ifo_file.monster_spawns);
                    npcs.extend(ifo_file.npcs);
                    event_objects.extend(ifo_file.event_objects);
                    ifo_count += 1;

                    min_x = u32::min(min_x, x);
                    min_y = u32::min(min_y, y);
                    max_x = u32::max(max_x, x);
                    max_y = u32::max(max_y, y);
                }
            }
        }

        let sector_size = STB_ZONE
            .get_zone_sector_size(index)
            .unwrap_or(0)
            .clamp(MIN_SECTOR_SIZE, MAX_SECTOR_SIZE) as u32;
        let block_size = 16.0 * zon_file.grid_per_patch * zon_file.grid_size;
        let num_blocks_x = (max_x + 1) - min_x;
        let num_blocks_y = (max_y + 1) - min_y;
        let num_sectors_x = ((num_blocks_x as f32 * block_size) / sector_size as f32) as u32;
        let num_sectors_y = ((num_blocks_y as f32 * block_size) / sector_size as f32) as u32;

        println!(
            "Loaded zone {}, blocks: {} monster spawns: {}, npcs: {}, sectors ({}, {})",
            index as u16,
            ifo_count,
            monster_spawns.len(),
            npcs.len(),
            num_sectors_x, num_sectors_y
        );
        Ok(Self {
            id: index as u16,
            sector_size,
            grid_per_patch: zon_file.grid_per_patch,
            grid_size: zon_file.grid_size,
            event_positions: zon_file.event_positions,
            monster_spawns,
            npcs,
            event_objects,
            sectors_base_position: Point2::new(
                (min_x as f32) * block_size,
                (min_y as f32) * block_size,
            ),
            num_sectors_x,
            num_sectors_y,
        })
    }
}
