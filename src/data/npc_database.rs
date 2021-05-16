use std::collections::HashMap;

use crate::data::item_database::ItemReference;

#[derive(Clone, Copy)]
pub struct NpcReference(pub usize);

#[derive(Clone)]
pub struct NpcConversationReference(pub String);

pub struct NpcData {
    pub walk_speed: u32,
    pub run_speed: u32,
    pub scale: f32,
    pub right_hand_part_index: u32,
    pub left_hand_part_index: u32,
    pub level: u32,
    pub health_points: u32,
    pub attack: u32,
    pub hit: u32,
    pub defence: u32,
    pub resistance: u32,
    pub avoid: u32,
    pub attack_speed: u32,
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
    pub attack_range: u32,
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
    pub npc_height: u32,
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
}
