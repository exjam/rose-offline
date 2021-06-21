use std::collections::HashMap;

use crate::{data::item_database::ItemReference, game::components::MotionData};

use super::MotionFileData;

#[derive(Clone, Copy)]
pub struct NpcReference(pub usize);

#[derive(Clone)]
pub struct NpcConversationReference(pub String);

#[derive(Hash, PartialEq, Eq)]
pub enum NpcMotionAction {
    Stop,
    Move,
    Attack,
    Hit,
    Die,
    Run,
    Cast1,
    SkillAction1,
    Cast2,
    SkillAction2,
    Etc,
}

pub struct NpcData {
    pub walk_speed: i32,
    pub run_speed: i32,
    pub scale: f32,
    pub right_hand_part_index: u32,
    pub left_hand_part_index: u32,
    pub level: i32,
    pub health_points: i32,
    pub attack: i32,
    pub hit: i32,
    pub defence: i32,
    pub resistance: i32,
    pub avoid: i32,
    pub attack_speed: i32,
    pub is_attack_magic_damage: bool,
    pub ai_file_index: u32,
    pub reward_xp: u32,
    pub drop_table_index: u32,
    pub drop_money: u32,
    pub drop_item: Option<ItemReference>,
    pub npc_minimap_icon_index: u32,
    pub summon_point_requirement: u32,
    pub shop_tabs: Vec<u32>,
    pub shop_union_number: u32,
    pub is_targetable: bool,
    pub attack_range: i32,
    pub npc_type_index: u32,
    pub hit_sound_index: u32,
    pub face_icon_index: u32,
    pub summon_monster_type: u32,
    pub normal_effect_sound_index: u32,
    pub attack_sound_index: u32,
    pub hitted_sound_index: u32,
    pub hand_hit_effect_index: u32,
    pub dead_effect_index: u32,
    pub die_sound_index: u32,
    pub npc_quest_type: u32,
    pub glow_colour: (f32, f32, f32),
    pub string_id: String,
    pub create_effect_index: u32,
    pub create_sound_index: u32,
    pub death_quest_trigger_name: String,
    pub npc_height: i32,
    pub motion_data: HashMap<NpcMotionAction, MotionFileData>,
}

pub struct NpcConversationData {
    pub index: usize,
    pub name: String,
    pub _type: String,
    pub description: String,
    pub filename: String,
}

pub struct NpcDatabase {
    npcs: HashMap<u16, NpcData>,
    conversation_files: HashMap<String, NpcConversationData>,
}

impl NpcDatabase {
    pub fn new(
        npcs: HashMap<u16, NpcData>,
        conversation_files: HashMap<String, NpcConversationData>,
    ) -> Self {
        Self {
            npcs,
            conversation_files,
        }
    }

    pub fn get_npc(&self, id: usize) -> Option<&NpcData> {
        self.npcs.get(&(id as u16))
    }

    pub fn get_conversation(&self, key: &NpcConversationReference) -> Option<&NpcConversationData> {
        self.conversation_files.get(&key.0)
    }

    pub fn get_npc_motions(&self, id: usize) -> MotionData {
        if let Some(npc) = self.get_npc(id) {
            MotionData {
                attack: npc.motion_data.get(&NpcMotionAction::Attack).cloned(),
            }
        } else {
            MotionData::default()
        }
    }
}
