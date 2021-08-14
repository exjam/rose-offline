use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroUsize},
    str::FromStr,
};

use crate::{
    data::{ItemReference, MotionFileData},
    game::components::{MotionData, MotionDataNpc},
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct NpcId(NonZeroU16);

id_wrapper_impl!(NpcId, NonZeroU16, u16);

#[derive(Clone, Debug)]
pub struct NpcConversationId(String);

id_wrapper_impl!(NpcConversationId, String);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct NpcStoreTabId(NonZeroU16);

id_wrapper_impl!(NpcStoreTabId, NonZeroU16, u16);

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
    pub id: NpcId,
    pub name: String,
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
    pub drop_money_rate: i32,
    pub drop_item_rate: i32,
    pub npc_minimap_icon_index: u32,
    pub summon_point_requirement: u32,
    pub store_tabs: [Option<NpcStoreTabId>; 4],
    pub store_union_number: Option<NonZeroUsize>,
    pub is_untargetable: bool,
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

pub struct NpcStoreTabData {
    pub name: String,
    pub items: HashMap<u16, ItemReference>,
}

pub struct NpcDatabase {
    npcs: HashMap<u16, NpcData>,
    conversation_files: HashMap<String, NpcConversationData>,
    store_tabs: HashMap<u16, NpcStoreTabData>,
}

impl NpcDatabase {
    pub fn new(
        npcs: HashMap<u16, NpcData>,
        conversation_files: HashMap<String, NpcConversationData>,
        store_tabs: HashMap<u16, NpcStoreTabData>,
    ) -> Self {
        Self {
            npcs,
            conversation_files,
            store_tabs,
        }
    }

    pub fn get_npc(&self, id: NpcId) -> Option<&NpcData> {
        self.npcs.get(&(id.get() as u16))
    }

    pub fn get_conversation(&self, key: &NpcConversationId) -> Option<&NpcConversationData> {
        self.conversation_files.get(&key.0)
    }

    pub fn get_npc_motions(&self, id: NpcId) -> MotionData {
        let npc_data = self.get_npc(id);
        let get_motion =
            |action| npc_data.and_then(|npc_data| npc_data.motion_data.get(&action).cloned());

        MotionData::with_npc_motions(MotionDataNpc {
            stop: get_motion(NpcMotionAction::Stop),
            walk: get_motion(NpcMotionAction::Move),
            attack: get_motion(NpcMotionAction::Attack),
            hit: get_motion(NpcMotionAction::Hit),
            die: get_motion(NpcMotionAction::Die),
            run: get_motion(NpcMotionAction::Run),
            cast1: get_motion(NpcMotionAction::Cast1),
            skill_action1: get_motion(NpcMotionAction::SkillAction1),
            cast2: get_motion(NpcMotionAction::Cast2),
            skill_action2: get_motion(NpcMotionAction::SkillAction2),
            etc: get_motion(NpcMotionAction::Etc),
        })
    }

    pub fn get_store_tab(&self, id: NpcStoreTabId) -> Option<&NpcStoreTabData> {
        self.store_tabs.get(&(id.get() as u16))
    }
}
