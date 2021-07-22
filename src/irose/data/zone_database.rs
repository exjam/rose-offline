use log::debug;
use nalgebra::{Point2, Point3, Vector3};
use std::{collections::HashMap, path::Path};

use crate::{
    data::{
        formats::{
            ifo::{self, IfoReadError, MonsterSpawn},
            FileReader, IfoFile, StbFile, VfsIndex, ZonFile, ZonReadError,
        },
        NpcConversationReference, NpcReference, ZoneData, ZoneDatabase, ZoneMonsterSpawnPoint,
        ZoneNpcSpawn,
    },
    stb_column,
};

const MIN_SECTOR_SIZE: u32 = 5000;
const MAX_SECTOR_SIZE: u32 = 12000;

pub struct StbZone(pub StbFile);

#[allow(dead_code)]
impl StbZone {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 1, get_zone_file, &str }
    stb_column! { 2, get_zone_start_event_position_name, &str }
    stb_column! { 3, get_zone_revive_event_position_name, &str }
    stb_column! { 0, get_zone_is_underground, bool }
    stb_column! { 5, get_zone_background_music_day, &str }
    stb_column! { 6, get_zone_background_music_night, &str }
    stb_column! { 7, get_zone_skybox_index, u32 }
    stb_column! { 8, get_zone_minimap_filename, &str }
    stb_column! { 9, get_zone_minimap_start_x, u32 }
    stb_column! { 10, get_zone_minimap_start_y, u32 }
    stb_column! { 11, get_zone_object_table, &str }
    stb_column! { 12, get_zone_cnst_table, &str }
    stb_column! { 13, get_zone_day_cycle_time, u32 }
    stb_column! { 14, get_zone_morning_time, u32 }
    stb_column! { 15, get_zone_day_time, u32 }
    stb_column! { 16, get_zone_evening_time, u32 }
    stb_column! { 17, get_zone_night_time, u32 }
    stb_column! { 18, get_zone_pvp_state, u32 }
    stb_column! { 19, get_zone_planet, u32 }
    stb_column! { 20, get_zone_footstep_type, u32 }
    stb_column! { 21, get_zone_camera_type, u32 }
    stb_column! { 22, get_zone_join_trigger, &str }
    stb_column! { 23, get_zone_kill_trigger, &str }
    stb_column! { 24, get_zone_dead_trigger, &str }
    stb_column! { 25, get_zone_sector_size, u32 }
    stb_column! { 26, get_zone_string_id, &str }
    stb_column! { 27, get_zone_weather_type, u32 }
    stb_column! { 28, get_zone_party_xp_a, u32 }
    stb_column! { 29, get_zone_party_xp_b, u32 }
    stb_column! { 30, get_zone_vehicle_use_flags, u32 }
    stb_column! { 31, get_zone_revive_zone_no, u32 }
    stb_column! { 32, get_zone_revive_pos_x, u32 }
    stb_column! { 33, get_zone_revive_pos_y, u32 }
}

pub enum LoadZoneError {
    NotExists,
    ZonFileInvalidPath,
    ZonFileNotFound,
    ZonFileInvalid,
    IfoFileInvalid,
}

impl From<ZonReadError> for LoadZoneError {
    fn from(_: ZonReadError) -> Self {
        Self::ZonFileInvalid
    }
}

impl From<IfoReadError> for LoadZoneError {
    fn from(_: IfoReadError) -> Self {
        Self::IfoFileInvalid
    }
}

impl From<&ifo::MonsterSpawnPoint> for ZoneMonsterSpawnPoint {
    fn from(spawn: &ifo::MonsterSpawnPoint) -> Self {
        let transform_spawn_list = |spawn_list: &Vec<MonsterSpawn>| {
            spawn_list
                .iter()
                .map(|ifo::MonsterSpawn { id, count }| {
                    (NpcReference(*id as usize), *count as usize)
                })
                .collect()
        };
        Self {
            position: spawn.object.position,
            basic_spawns: transform_spawn_list(&spawn.basic_spawns),
            tactic_spawns: transform_spawn_list(&spawn.tactic_spawns),
            interval: spawn.interval,
            limit_count: spawn.limit_count,
            range: spawn.range,
            tactic_points: spawn.tactic_points,
        }
    }
}

fn create_monster_spawn(
    spawn: &ifo::MonsterSpawnPoint,
    object_offset: Vector3<f32>,
) -> ZoneMonsterSpawnPoint {
    let transform_spawn_list = |spawn_list: &Vec<MonsterSpawn>| {
        spawn_list
            .iter()
            .map(|ifo::MonsterSpawn { id, count }| (NpcReference(*id as usize), *count as usize))
            .collect()
    };

    ZoneMonsterSpawnPoint {
        position: spawn.object.position + object_offset,
        basic_spawns: transform_spawn_list(&spawn.basic_spawns),
        tactic_spawns: transform_spawn_list(&spawn.tactic_spawns),
        interval: spawn.interval,
        limit_count: spawn.limit_count,
        range: spawn.range,
        tactic_points: spawn.tactic_points,
    }
}

fn create_npc_spawn(npc: &ifo::Npc, object_offset: Vector3<f32>) -> ZoneNpcSpawn {
    ZoneNpcSpawn {
        npc: NpcReference(npc.object.object_id as usize),
        position: npc.object.position + object_offset,
        direction: npc.object.rotation.euler_angles().2.to_degrees(),
        conversation: NpcConversationReference(npc.quest_file_name.to_string()),
    }
}

fn load_zone(vfs: &VfsIndex, data: &StbZone, id: usize) -> Result<ZoneData, LoadZoneError> {
    let zone_file =
        VfsIndex::normalise_path(data.get_zone_file(id).ok_or(LoadZoneError::NotExists)?);
    let zone_base_directory = Path::new(&zone_file)
        .parent()
        .ok_or(LoadZoneError::ZonFileInvalidPath)?;

    let file = vfs
        .open_file(&zone_file)
        .ok_or(LoadZoneError::ZonFileNotFound)?;
    let zon_file = ZonFile::read(FileReader::from(&file))?;

    let mut monster_spawns = Vec::new();
    let mut npcs = Vec::new();
    let mut ifo_count = 0;

    let mut min_x = None;
    let mut min_y = None;
    let mut max_x = None;
    let mut max_y = None;

    let objects_offset = Vector3::new(
        (64.0 / 2.0) * (zon_file.grid_size * zon_file.grid_per_patch * 16.0)
            + (zon_file.grid_size * zon_file.grid_per_patch * 16.0) / 2.0,
        (64.0 / 2.0) * (zon_file.grid_size * zon_file.grid_per_patch * 16.0)
            + (zon_file.grid_size * zon_file.grid_per_patch * 16.0) / 2.0,
        0.0,
    );

    for y in 0..64u32 {
        for x in 0..64u32 {
            let ifo_file_path = zone_base_directory.join(format!("{}_{}.IFO", x, y));
            if let Some(file) = vfs.open_file(&ifo_file_path.to_string_lossy()) {
                let ifo_file = IfoFile::read(FileReader::from(&file))?;
                monster_spawns.extend(
                    ifo_file
                        .monster_spawns
                        .iter()
                        .map(|x| create_monster_spawn(x, objects_offset)),
                );
                npcs.extend(
                    ifo_file
                        .npcs
                        .iter()
                        .map(|x| create_npc_spawn(x, objects_offset)),
                );
                ifo_count += 1;

                min_x = Some(min_x.map_or(x, |value| u32::min(value, x)));
                min_y = Some(min_y.map_or(y, |value| u32::min(value, y)));
                max_x = Some(max_x.map_or(x, |value| u32::max(value, x)));
                max_y = Some(max_y.map_or(y, |value| u32::max(value, y)));
            }
        }
    }

    if min_x.is_none() || min_y.is_none() || max_x.is_none() || max_y.is_none() {
        return Err(LoadZoneError::NotExists);
    }

    let min_x = min_x.unwrap();
    let min_y = min_y.unwrap() - 1; // Map grows in negative y
    let max_x = max_x.unwrap() + 1; // Map grows in positive x
    let max_y = max_y.unwrap();

    let sector_size = data
        .get_zone_sector_size(id)
        .unwrap_or(0)
        .clamp(MIN_SECTOR_SIZE, MAX_SECTOR_SIZE);
    let block_size = 16.0 * zon_file.grid_per_patch * zon_file.grid_size;
    let num_blocks_x = max_x - min_x;
    let num_blocks_y = max_y - min_y;
    let num_sectors_x = ((num_blocks_x as f32 * block_size) / sector_size as f32) as u32;
    let num_sectors_y = ((num_blocks_y as f32 * block_size) / sector_size as f32) as u32;

    let start_event_position_name = data.get_zone_start_event_position_name(id).unwrap_or("");
    let revive_event_position_name = data.get_zone_revive_event_position_name(id).unwrap_or("");
    let mut start_position = Point3::new(0.0, 0.0, 0.0);
    let mut revive_positions = Vec::new();
    for (name, mut position) in zon_file.event_positions.iter() {
        let y = position.y;
        position.y = position.z;
        position.z = y;
        position += objects_offset;

        if name == start_event_position_name {
            start_position = position;
        }

        if name == revive_event_position_name {
            revive_positions.push(position);
        }
    }

    debug!(
        "Loaded zone {}, blocks: {} monster spawns: {}, npcs: {}, sectors ({}, {}), start: {}",
        id,
        ifo_count,
        monster_spawns.len(),
        npcs.len(),
        num_sectors_x,
        num_sectors_y,
        start_position.xy(),
    );
    Ok(ZoneData {
        id: id as u16,
        sector_size,
        grid_per_patch: zon_file.grid_per_patch,
        grid_size: zon_file.grid_size,
        monster_spawns,
        npcs,
        sectors_base_position: Point2::new(
            (min_x as f32) * block_size,
            (min_y as f32) * block_size,
        ),
        num_sectors_x,
        num_sectors_y,
        start_position,
        revive_positions,
    })
}

pub fn get_zone_database(vfs: &VfsIndex) -> Option<ZoneDatabase> {
    let file = vfs.open_file("3DDATA/STB/LIST_ZONE.STB")?;
    let data = StbZone(StbFile::read(FileReader::from(&file)).ok()?);
    let mut zones = HashMap::new();
    for id in 0..data.rows() {
        let zone_file = data.get_zone_file(id);
        if zone_file.is_none() {
            continue;
        }

        if let Ok(zone_data) = load_zone(vfs, &data, id) {
            zones.insert(id as u16, zone_data);
        }
    }

    Some(ZoneDatabase::new(zones))
}
