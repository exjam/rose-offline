use enum_map::enum_map;
use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroUsize},
    sync::Arc,
};

use rose_data::{
    EffectFileId, EffectId, MotionFileData, MotionId, NpcConversationData, NpcData, NpcDatabase,
    NpcDatabaseOptions, NpcId, NpcMotionAction, NpcStoreTabData, NpcStoreTabId, SoundId,
    StringDatabase,
};
use rose_file_readers::{stb_column, ChrFile, StbFile, VfsPathBuf, VirtualFilesystem, ZmoFile};

use crate::data_decoder::decode_item_base1000;

struct StbNpc(StbFile);

impl StbNpc {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 2, get_walk_speed, i32 }
    stb_column! { 3, get_run_speed, i32 }
    stb_column! { 4, get_scale, u32 }
    stb_column! { 5, get_right_hand_part_index, u32 }
    stb_column! { 6, get_left_hand_part_index, u32 }
    stb_column! { 7, get_level, i32 }
    stb_column! { 8, get_health_points, i32 }
    stb_column! { 9, get_attack, i32 }
    stb_column! { 10, get_hit, i32 }
    stb_column! { 11, get_defence, i32 }
    stb_column! { 12, get_resistance, i32 }
    stb_column! { 13, get_avoid, i32 }
    stb_column! { 14, get_attack_speed, i32 }
    stb_column! { 15, get_attack_is_magic_damage, bool }
    stb_column! { 16, get_ai_file_index, u32 }
    stb_column! { 17, get_reward_xp, u32 }
    stb_column! { 18, get_drop_table_index, u32 }
    stb_column! { 18, get_npc_minimap_icon_index, u32 }
    stb_column! { 19, get_drop_money_rate, i32 }
    stb_column! { 20, get_drop_item_rate, i32 }
    stb_column! { 20, get_store_union_number, NonZeroUsize }
    stb_column! { 21, get_summon_point_requirement, u32 }
    stb_column! { 21..=24, get_store_tabs, [Option<NpcStoreTabId>; 4] }
    stb_column! { 25, get_is_untargetable, bool }
    stb_column! { 26, get_attack_range, i32 }
    stb_column! { 27, get_npc_type_index, NonZeroU16 }
    stb_column! { 28, get_hit_sound_material_type, u32 }
    stb_column! { 29, get_face_icon_index, u32 }
    stb_column! { 29, get_summon_monster_type, u32 }
    stb_column! { 30, get_normal_effect_sound_id, SoundId }
    stb_column! { 31, get_attack_sound_id, SoundId }
    stb_column! { 32, get_hitted_sound_id, SoundId }
    stb_column! { 33, get_hand_hit_effect_id, EffectId }
    stb_column! { 34, get_die_effect_file_id, EffectFileId }
    stb_column! { 35, get_die_sound_id, SoundId }
    stb_column! { 38, get_npc_quest_type, u32 }

    pub fn get_glow_colour(&self, id: usize) -> (f32, f32, f32) {
        let mut colour = self.0.try_get_int(id, 39).unwrap_or(0);

        let red = colour / 1000000;
        colour %= 1000000;

        let green = colour / 1000;
        colour %= 1000;

        let blue = colour;

        (
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
        )
    }

    stb_column! { 40, get_string_id, &str }
    stb_column! { 41, get_death_quest_trigger_name, &str }
    stb_column! { 42, get_npc_height, i32 }
    stb_column! { 44, get_create_effect_index, u32 }
    stb_column! { 45, get_create_sound_id, SoundId }
}

pub struct StbEvent(pub StbFile);

#[allow(dead_code)]
impl StbEvent {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    pub fn get_row_key(&self, row: usize) -> Option<&str> {
        self.0.try_get_row_name(row)
    }

    stb_column! { 0, get_name, &str }
    stb_column! { 1, get_type, &str }
    stb_column! { 2, get_description, &str }
    stb_column! { 3, get_filename, &str }
}

fn load_motion_file_data(
    vfs: &VirtualFilesystem,
    path: &str,
    options: &NpcDatabaseOptions,
) -> Option<MotionFileData> {
    if path.is_empty() {
        return None;
    }
    let path = VfsPathBuf::new(path);

    if options.load_frame_data {
        let zmo = vfs.read_file::<ZmoFile, _>(&path).ok()?;
        Some(MotionFileData {
            path,
            duration: zmo.get_duration(),
            total_attack_frames: zmo.total_attack_frames,
        })
    } else {
        Some(MotionFileData {
            path,
            ..Default::default()
        })
    }
}

pub fn get_npc_database(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
    options: &NpcDatabaseOptions,
) -> Result<NpcDatabase, anyhow::Error> {
    let model_data = vfs.read_file::<ChrFile, _>("3DDATA/NPC/LIST_NPC.CHR")?;

    let data = StbNpc(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_NPC.STB")?);

    let mut motions = Vec::with_capacity(model_data.motion_files.len());
    for path in model_data.motion_files.iter() {
        motions.push(load_motion_file_data(vfs, path, options));
    }

    let mut npcs = Vec::with_capacity(data.rows());
    let mut npc_count = 0;
    npcs.push(None); // NPC id 0
    for id in 1..data.rows() {
        let npc_model_data = model_data.npcs.get(&(id as u16));
        let npc_string_id = data.get_string_id(id);
        let ai_file_index = data.get_ai_file_index(id).unwrap_or(0);

        if npc_string_id.is_none() && npc_model_data.is_none() && ai_file_index == 0 {
            // NPC must have either a name, a model, or an AI file
            npcs.push(None);
            continue;
        }

        let mut motion_data = Vec::new();
        if let Some(npc_model_data) = npc_model_data {
            for &(motion_id, motion_file_index) in npc_model_data.motion_ids.iter() {
                if let Some(motion_file_data) = motions
                    .get(motion_file_index as usize)
                    .and_then(|x| x.as_ref())
                {
                    motion_data.push((MotionId::new(motion_id), motion_file_data.clone()));
                }
            }
        }

        let name = if let Some(entry) = npc_string_id.and_then(|key| string_database.get_npc(key)) {
            unsafe { std::mem::transmute(entry.text) }
        } else {
            ""
        };

        npc_count += 1;
        npcs.push(Some(NpcData {
            id: NpcId::new(id as u16).unwrap(),
            name,
            walk_speed: data.get_walk_speed(id).unwrap_or(0),
            run_speed: data.get_run_speed(id).unwrap_or(0),
            scale: (data.get_scale(id).unwrap_or(100) as f32) / 100.0,
            right_hand_part_index: data.get_right_hand_part_index(id).unwrap_or(0),
            left_hand_part_index: data.get_left_hand_part_index(id).unwrap_or(0),
            level: data.get_level(id).unwrap_or(0),
            health_points: data.get_health_points(id).unwrap_or(0),
            attack: data.get_attack(id).unwrap_or(0),
            hit: data.get_hit(id).unwrap_or(0),
            defence: data.get_defence(id).unwrap_or(0),
            resistance: data.get_resistance(id).unwrap_or(0),
            avoid: data.get_avoid(id).unwrap_or(0),
            attack_speed: data.get_attack_speed(id).unwrap_or(0),
            is_attack_magic_damage: data.get_attack_is_magic_damage(id).unwrap_or(false),
            ai_file_index: data.get_ai_file_index(id).unwrap_or(0),
            reward_xp: data.get_reward_xp(id).unwrap_or(0),
            drop_table_index: data.get_drop_table_index(id).unwrap_or(0),
            drop_money_rate: data.get_drop_money_rate(id).unwrap_or(0),
            drop_item_rate: data.get_drop_item_rate(id).unwrap_or(0),
            npc_minimap_icon_index: data.get_npc_minimap_icon_index(id).unwrap_or(0),
            summon_point_requirement: data.get_summon_point_requirement(id).unwrap_or(0),
            store_tabs: data.get_store_tabs(id),
            store_union_number: data.get_store_union_number(id),
            is_untargetable: data.get_is_untargetable(id).unwrap_or(false),
            attack_range: data.get_attack_range(id).unwrap_or(0),
            npc_type_index: data.get_npc_type_index(id),
            hit_sound_material_type: data.get_hit_sound_material_type(id).unwrap_or(0),
            face_icon_index: data.get_face_icon_index(id).unwrap_or(0),
            summon_monster_type: data.get_summon_monster_type(id).unwrap_or(0),
            normal_effect_sound_id: data.get_normal_effect_sound_id(id),
            attack_sound_id: data.get_attack_sound_id(id),
            hitted_sound_id: data.get_hitted_sound_id(id),
            hand_hit_effect_id: data.get_hand_hit_effect_id(id),
            die_effect_file_id: data.get_die_effect_file_id(id),
            die_sound_id: data.get_die_sound_id(id),
            npc_quest_type: data.get_npc_quest_type(id).unwrap_or(0),
            glow_colour: data.get_glow_colour(id),
            create_effect_index: data.get_create_effect_index(id).unwrap_or(0),
            create_sound_id: data.get_create_sound_id(id),
            death_quest_trigger_name: data
                .get_death_quest_trigger_name(id)
                .unwrap_or("")
                .to_string(),
            npc_height: data.get_npc_height(id).unwrap_or(0),
            motion_data,
        }));
    }

    let data = StbEvent(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_EVENT.STB")?);
    let mut conversation_files = HashMap::new();
    for id in 0..data.rows() {
        let key = data.get_row_key(id);
        let filename = data.get_filename(id);
        if key.is_none() || filename.is_none() {
            continue;
        }
        conversation_files.insert(
            key.unwrap().to_string(),
            NpcConversationData {
                index: id,
                name: data.get_name(id).unwrap_or("").to_string(),
                _type: data.get_type(id).unwrap_or("").to_string(),
                description: data.get_description(id).unwrap_or("").to_string(),
                filename: filename.unwrap().to_string(),
            },
        );
    }

    let data = vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_SELL.STB")?;
    let mut store_tabs = HashMap::new();
    for id in 1..data.rows() {
        let mut items = HashMap::new();
        for col in 2..data.columns() {
            if let Some(item) = decode_item_base1000(data.get_int(id, col) as usize) {
                items.insert((col - 2) as u16, item);
            }
        }

        if !items.is_empty() {
            let name = if let Some(entry) = string_database.get_npc_store_tab(data.get(id, 1)) {
                unsafe { std::mem::transmute(entry.text) }
            } else {
                ""
            };

            store_tabs.insert(
                NpcStoreTabId::new(id as u16).unwrap(),
                NpcStoreTabData { name, items },
            );
        }
    }

    let action_map = enum_map! {
        NpcMotionAction::Stop => MotionId::new(0),
        NpcMotionAction::Move => MotionId::new(1),
        NpcMotionAction::Attack => MotionId::new(2),
        NpcMotionAction::Hit => MotionId::new(3),
        NpcMotionAction::Die => MotionId::new(4),
        NpcMotionAction::Run => MotionId::new(5),
        NpcMotionAction::Cast1 => MotionId::new(6),
        NpcMotionAction::SkillAction1 => MotionId::new(7),
        NpcMotionAction::Cast2 => MotionId::new(8),
        NpcMotionAction::SkillAction2 => MotionId::new(9),
        NpcMotionAction::Etc => MotionId::new(10),
    };

    log::debug!(
        "Loaded {} NPCs, {} motions, {} conversations, {} stores",
        npc_count,
        motions.len(),
        conversation_files.len(),
        store_tabs.len()
    );
    Ok(NpcDatabase::new(
        string_database,
        npcs,
        conversation_files,
        store_tabs,
        action_map,
    ))
}
