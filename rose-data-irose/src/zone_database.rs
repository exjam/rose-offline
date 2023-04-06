use std::sync::Arc;

use bevy::math::{Quat, Vec2, Vec3, Vec3Swizzles};
use log::debug;

use rose_data::{
    NpcConversationId, NpcId, SkyboxId, StringDatabase, ZoneData, ZoneDatabase, ZoneEventObject,
    ZoneId, ZoneList, ZoneListEntry, ZoneMonsterSpawnPoint, ZoneNpcSpawn, WORLD_TICKS_PER_DAY,
};
use rose_file_readers::{
    stb_column, IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc,
    IfoReadOptions, StbFile, VfsPath, VfsPathBuf, VirtualFilesystem, ZonFile, ZonReadOptions,
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
    stb_column! { 7, get_zone_skybox_id, SkyboxId }
    stb_column! { 8, get_zone_minimap_filename, &str }
    stb_column! { 9, get_zone_minimap_start_x, u32 }
    stb_column! { 10, get_zone_minimap_start_y, u32 }
    stb_column! { 11, get_zone_deco_table, &str }
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
}

fn create_monster_spawn(
    spawn: &IfoMonsterSpawnPoint,
    object_offset: Vec3,
) -> ZoneMonsterSpawnPoint {
    let transform_spawn_list = |spawn_list: &Vec<IfoMonsterSpawn>| {
        spawn_list
            .iter()
            .map(|&IfoMonsterSpawn { id, count }| (NpcId::new(id as u16).unwrap(), count as usize))
            .collect()
    };

    ZoneMonsterSpawnPoint {
        position: Vec3::new(
            spawn.object.position.x,
            spawn.object.position.y,
            spawn.object.position.z,
        ) + object_offset,
        basic_spawns: transform_spawn_list(&spawn.basic_spawns),
        tactic_spawns: transform_spawn_list(&spawn.tactic_spawns),
        interval: spawn.interval,
        limit_count: spawn.limit_count,
        range: spawn.range,
        tactic_points: spawn.tactic_points,
    }
}

fn create_npc_spawn(npc: &IfoNpc, object_offset: Vec3) -> ZoneNpcSpawn {
    ZoneNpcSpawn {
        npc_id: NpcId::new(npc.object.object_id as u16).unwrap(),
        position: Vec3::new(
            npc.object.position.x,
            npc.object.position.y,
            npc.object.position.z,
        ) + object_offset,
        direction: Quat::from_xyzw(
            npc.object.rotation.x,
            npc.object.rotation.y,
            npc.object.rotation.z,
            npc.object.rotation.w,
        )
        .to_euler(bevy::math::EulerRot::XYZ)
        .2
        .to_degrees(),
        conversation: NpcConversationId::new(npc.quest_file_name.to_string()),
    }
}

fn create_event_object(
    event_object: &IfoEventObject,
    object_offset: Vec3,
    map_chunk_x: i32,
    map_chunk_y: i32,
) -> ZoneEventObject {
    ZoneEventObject {
        event_id: event_object.object.event_id,
        map_chunk_x,
        map_chunk_y,
        position: Vec3::new(
            event_object.object.position.x,
            event_object.object.position.y,
            event_object.object.position.z,
        ) + object_offset,
    }
}

fn load_zone(
    vfs: &VirtualFilesystem,
    data: &StbZone,
    string_database: &StringDatabase,
    id: usize,
) -> Result<ZoneData, LoadZoneError> {
    let zone_file = VfsPath::from(data.get_zone_file(id).ok_or(LoadZoneError::NotExists)?);
    let zone_base_directory = zone_file
        .path()
        .parent()
        .ok_or(LoadZoneError::ZonFileInvalidPath)?;

    let zon_file: ZonFile = vfs
        .read_file_with(
            &zone_file,
            &ZonReadOptions {
                skip_zone_info: false,
                skip_event_positions: false,
                skip_textures: true,
                skip_tiles: true,
            },
        )
        .map_err(|_| LoadZoneError::ZonFileNotFound)?;

    let mut monster_spawns = Vec::new();
    let mut npcs = Vec::new();
    let mut event_objects = Vec::new();

    let mut num_blocks = 0;
    let mut min_block_x = None;
    let mut min_block_y = None;
    let mut max_block_x = None;
    let mut max_block_y = None;

    let objects_offset = Vec3::new(
        (64.0 / 2.0) * (zon_file.grid_size * zon_file.grid_per_patch * 16.0)
            + (zon_file.grid_size * zon_file.grid_per_patch * 16.0) / 2.0,
        (64.0 / 2.0) * (zon_file.grid_size * zon_file.grid_per_patch * 16.0)
            + (zon_file.grid_size * zon_file.grid_per_patch * 16.0) / 2.0,
        0.0,
    );

    let ifo_read_options = IfoReadOptions {
        skip_event_objects: false,
        skip_monster_spawns: false,
        skip_npcs: false,
        skip_animated_objects: true,
        skip_collision_objects: true,
        skip_cnst_objects: true,
        skip_deco_objects: true,
        skip_effect_objects: true,
        skip_sound_objects: true,
        skip_water_planes: true,
        skip_warp_objects: true,
    };

    for block_y in 0..64u32 {
        for block_x in 0..64u32 {
            if let Ok(ifo_file) = vfs.read_file_with::<IfoFile, _>(
                zone_base_directory.join(format!("{}_{}.IFO", block_x, block_y)),
                &ifo_read_options,
            ) {
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
                event_objects.extend(ifo_file.event_objects.iter().map(|event_object| {
                    create_event_object(
                        event_object,
                        objects_offset,
                        block_x as i32,
                        block_y as i32,
                    )
                }));
                num_blocks += 1;

                min_block_x = Some(min_block_x.map_or(block_x, |value| u32::min(value, block_x)));
                min_block_y = Some(min_block_y.map_or(block_y, |value| u32::min(value, block_y)));
                max_block_x = Some(max_block_x.map_or(block_x, |value| u32::max(value, block_x)));
                max_block_y = Some(max_block_y.map_or(block_y, |value| u32::max(value, block_y)));
            }
        }
    }

    if min_block_x.is_none()
        || min_block_y.is_none()
        || max_block_x.is_none()
        || max_block_y.is_none()
    {
        return Err(LoadZoneError::NotExists);
    }

    let min_x = min_block_x.unwrap();
    let min_y = min_block_y.unwrap() + 1;
    let max_x = max_block_x.unwrap() + 1;
    let max_y = max_block_y.unwrap() + 2;

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
    let mut start_position = Vec3::new(0.0, 0.0, 0.0);
    let mut revive_positions = Vec::new();
    for (name, position) in zon_file.event_positions.iter() {
        let position = Vec3::new(position.x, position.y, position.z).xzy() + objects_offset;

        if name == start_event_position_name {
            start_position = position;
        }

        if name == revive_event_position_name {
            revive_positions.push(position);
        }
    }

    let zone_strings = string_database.get_zone(data.get_zone_string_id(id).unwrap_or(""));
    let name = zone_strings
        .as_ref()
        .map_or("", |x| unsafe { std::mem::transmute(x.name) });
    let description = zone_strings
        .as_ref()
        .map_or("", |x| unsafe { std::mem::transmute(x.description) });
    debug!(
        "Loaded zone {} {} blocks: {}, spawns: {}, npcs: {}, sectors ({}, {}), start: {}",
        id,
        name,
        num_blocks,
        monster_spawns.len(),
        npcs.len(),
        num_sectors_x,
        num_sectors_y,
        start_position.xy(),
    );
    Ok(ZoneData {
        id: ZoneId::new(id as u16).unwrap(),
        name,
        description,
        sector_size,
        grid_per_patch: zon_file.grid_per_patch,
        grid_size: zon_file.grid_size,
        event_objects,
        monster_spawns,
        npcs,
        sectors_base_position: Vec2::new((min_x as f32) * block_size, (min_y as f32) * block_size),
        num_sectors_x,
        num_sectors_y,
        start_position,
        revive_positions,
        event_positions: zon_file
            .event_positions
            .into_iter()
            .map(|(name, position)| {
                (
                    name,
                    Vec3::new(position.x, position.y, position.z).xzy() + objects_offset,
                )
            })
            .collect(),
        day_cycle: data
            .get_zone_day_cycle_time(id)
            .unwrap_or(WORLD_TICKS_PER_DAY as u32),
        morning_time: data
            .get_zone_morning_time(id)
            .unwrap_or((WORLD_TICKS_PER_DAY / 6) as u32),
        day_time: data
            .get_zone_day_time(id)
            .unwrap_or((2 * WORLD_TICKS_PER_DAY / 6) as u32),
        evening_time: data
            .get_zone_evening_time(id)
            .unwrap_or((4 * WORLD_TICKS_PER_DAY / 6) as u32),
        night_time: data
            .get_zone_night_time(id)
            .unwrap_or((5 * WORLD_TICKS_PER_DAY / 6) as u32),
        skybox_id: data.get_zone_skybox_id(id),
    })
}

pub fn get_zone_database(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
) -> Result<ZoneDatabase, anyhow::Error> {
    let data = StbZone(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_ZONE.STB")?);
    let mut zones = Vec::with_capacity(data.rows());
    zones.push(None); // Zone ID 0
    for id in 1..data.rows() {
        zones.push(load_zone(vfs, &data, &string_database, id).ok());
    }

    Ok(ZoneDatabase::new(string_database, zones))
}

fn load_zone_list_entry(
    data: &StbZone,
    string_database: &StringDatabase,
    id: usize,
) -> Result<ZoneListEntry, LoadZoneError> {
    let zone_strings = string_database.get_zone(data.get_zone_string_id(id).unwrap_or(""));

    Ok(ZoneListEntry {
        id: ZoneId::new(id as u16).unwrap(),
        name: zone_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.name) }),
        description: zone_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.description) }),
        minimap_path: data.get_zone_minimap_filename(id).map(VfsPathBuf::new),
        minimap_start_x: data.get_zone_minimap_start_x(id).unwrap_or(0),
        minimap_start_y: data.get_zone_minimap_start_y(id).unwrap_or(0),
        zon_file_path: VfsPathBuf::new(data.get_zone_file(id).ok_or(LoadZoneError::NotExists)?),
        zsc_cnst_path: VfsPathBuf::new(data.get_zone_cnst_table(id).unwrap_or("")),
        zsc_deco_path: VfsPathBuf::new(data.get_zone_deco_table(id).unwrap_or("")),
        day_cycle: data
            .get_zone_day_cycle_time(id)
            .unwrap_or(WORLD_TICKS_PER_DAY as u32),
        morning_time: data
            .get_zone_morning_time(id)
            .unwrap_or((WORLD_TICKS_PER_DAY / 6) as u32),
        day_time: data
            .get_zone_day_time(id)
            .unwrap_or((2 * WORLD_TICKS_PER_DAY / 6) as u32),
        evening_time: data
            .get_zone_evening_time(id)
            .unwrap_or((4 * WORLD_TICKS_PER_DAY / 6) as u32),
        night_time: data
            .get_zone_night_time(id)
            .unwrap_or((5 * WORLD_TICKS_PER_DAY / 6) as u32),
        skybox_id: data.get_zone_skybox_id(id),
        background_music_day: data.get_zone_background_music_day(id).map(VfsPathBuf::new),
        background_music_night: data
            .get_zone_background_music_night(id)
            .map(VfsPathBuf::new),
        footstep_type: data.get_zone_footstep_type(id),
    })
}

pub fn get_zone_list(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
) -> Result<ZoneList, anyhow::Error> {
    let data = StbZone(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_ZONE.STB")?);
    let mut zones = Vec::with_capacity(data.rows());
    zones.push(None); // Zone ID 0
    for id in 1..data.rows() {
        zones.push(load_zone_list_entry(&data, &string_database, id).ok());
    }

    Ok(ZoneList::new(string_database, zones))
}
