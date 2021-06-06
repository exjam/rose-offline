use std::collections::HashMap;

use crate::{
    data::{
        formats::{ChrFile, FileReader, StbFile, VfsIndex, ZmoFile},
        NpcConversationData, NpcData, NpcDatabase, NpcMotionAction, NpcMotionData,
    },
    stb_column,
};

use super::decode_item_reference;

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
    stb_column! { 19, get_drop_money, u32 }
    stb_column! { 20, get_drop_item, u32 }
    stb_column! { 20, get_shop_union_number, u32 }
    stb_column! { 21, get_summon_point_requirement, u32 }

    pub fn get_shop_tabs(&self, id: usize) -> Option<Vec<u32>> {
        let mut tabs = Vec::new();
        for _ in 21..=24 {
            let tab = self.0.try_get_int(id, 21)? as u32;
            if tab != 0 {
                tabs.push(tab);
            }
        }
        Some(tabs)
    }

    stb_column! { 25, get_is_targetable, bool }
    stb_column! { 26, get_attack_range, i32 }
    stb_column! { 27, get_npc_type_index, u32 }
    stb_column! { 28, get_hit_sound_index, u32 }
    stb_column! { 29, get_face_icon_index, u32 }
    stb_column! { 29, get_summon_monster_type, u32 }
    stb_column! { 30, get_normal_effect_sound_index, u32 }
    stb_column! { 31, get_attack_sound_index, u32 }
    stb_column! { 32, get_hitted_sound_index, u32 }
    stb_column! { 33, get_hand_hit_effect_index, u32 }
    stb_column! { 34, get_dead_effect_index, u32 }
    stb_column! { 35, get_die_sound_index, u32 }
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
    stb_column! { 45, get_create_sound_index, u32 }
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

fn get_npc_action(action_index: u16) -> Option<NpcMotionAction> {
    match action_index {
        0 => Some(NpcMotionAction::Stop),
        1 => Some(NpcMotionAction::Move),
        2 => Some(NpcMotionAction::Attack),
        3 => Some(NpcMotionAction::Hit),
        4 => Some(NpcMotionAction::Die),
        5 => Some(NpcMotionAction::Run),
        6 => Some(NpcMotionAction::Cast1),
        7 => Some(NpcMotionAction::SkillAction1),
        8 => Some(NpcMotionAction::Cast2),
        9 => Some(NpcMotionAction::SkillAction2),
        10 => Some(NpcMotionAction::Etc),
        _ => None,
    }
}

pub fn get_npc_database(vfs: &VfsIndex) -> Option<NpcDatabase> {
    let file = vfs.open_file("3DDATA/NPC/LIST_NPC.CHR")?;
    let model_data = ChrFile::read(FileReader::from(&file)).ok()?;

    let file = vfs.open_file("3DDATA/STB/LIST_NPC.STB")?;
    let data = StbNpc(StbFile::read(FileReader::from(&file)).ok()?);

    let mut npcs = HashMap::new();
    for id in 1..data.rows() {
        if data.get_string_id(id).is_none() {
            continue;
        }

        let mut motion_data = Vec::new();
        if let Some(npc_model_data) = model_data.npcs.get(&(id as u16)) {
            for (action, motion_index) in npc_model_data.motion_ids.iter() {
                if let Some(action) = get_npc_action(*action) {
                    if let Some(file) = model_data
                        .motion_files
                        .get(*motion_index as usize)
                        .and_then(|path| vfs.open_file(path))
                    {
                        if let Ok(zmo) = ZmoFile::read(FileReader::from(&file)) {
                            motion_data.push(NpcMotionData {
                                action,
                                duration: zmo.get_duration(),
                                total_attack_frames: zmo.total_attack_frames,
                            });
                        }
                    }
                }
            }
        }

        npcs.insert(
            id as u16,
            NpcData {
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
                drop_money: data.get_drop_money(id).unwrap_or(0),
                drop_item: decode_item_reference(data.get_drop_item(id).unwrap_or(0)).ok(),
                npc_minimap_icon_index: data.get_npc_minimap_icon_index(id).unwrap_or(0),
                summon_point_requirement: data.get_summon_point_requirement(id).unwrap_or(0),
                shop_tabs: data.get_shop_tabs(id).unwrap_or_else(Vec::new),
                shop_union_number: data.get_shop_union_number(id).unwrap_or(0),
                is_targetable: data.get_is_targetable(id).unwrap_or(false),
                attack_range: data.get_attack_range(id).unwrap_or(0),
                npc_type_index: data.get_npc_type_index(id).unwrap_or(0),
                hit_sound_index: data.get_hit_sound_index(id).unwrap_or(0),
                face_icon_index: data.get_face_icon_index(id).unwrap_or(0),
                summon_monster_type: data.get_summon_monster_type(id).unwrap_or(0),
                normal_effect_sound_index: data.get_normal_effect_sound_index(id).unwrap_or(0),
                attack_sound_index: data.get_attack_sound_index(id).unwrap_or(0),
                hitted_sound_index: data.get_hitted_sound_index(id).unwrap_or(0),
                hand_hit_effect_index: data.get_hand_hit_effect_index(id).unwrap_or(0),
                dead_effect_index: data.get_dead_effect_index(id).unwrap_or(0),
                die_sound_index: data.get_die_sound_index(id).unwrap_or(0),
                npc_quest_type: data.get_npc_quest_type(id).unwrap_or(0),
                glow_colour: data.get_glow_colour(id),
                string_id: data.get_string_id(id).unwrap().to_string(),
                create_effect_index: data.get_create_effect_index(id).unwrap_or(0),
                create_sound_index: data.get_create_sound_index(id).unwrap_or(0),
                death_quest_trigger_name: data
                    .get_death_quest_trigger_name(id)
                    .unwrap_or(&"")
                    .to_string(),
                npc_height: data.get_npc_height(id).unwrap_or(0),
                motion_data,
            },
        );
    }

    let file = vfs.open_file("3DDATA/STB/LIST_EVENT.STB")?;
    let data = StbEvent(StbFile::read(FileReader::from(&file)).ok()?);
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
                name: data.get_name(id).unwrap_or(&"").to_string(),
                _type: data.get_type(id).unwrap_or(&"").to_string(),
                description: data.get_description(id).unwrap_or(&"").to_string(),
                filename: filename.unwrap().to_string(),
            },
        );
    }

    Some(NpcDatabase::new(npcs, conversation_files))
}
