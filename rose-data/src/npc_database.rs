use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroUsize},
    str::FromStr,
    sync::Arc,
};

use crate::{
    EffectFileId, EffectId, ItemReference, MotionFileData, MotionId, SoundId, StringDatabase,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpcId(NonZeroU16);

id_wrapper_impl!(NpcId, NonZeroU16, u16);

#[derive(Clone, Debug)]
pub struct NpcConversationId(String);

id_wrapper_impl!(NpcConversationId, String);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct NpcStoreTabId(NonZeroU16);

id_wrapper_impl!(NpcStoreTabId, NonZeroU16, u16);

#[derive(Copy, Clone, Debug, Enum)]
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
    pub name: &'static str,
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
    pub hit_sound_material_type: u32,
    pub face_icon_index: u32,
    pub summon_monster_type: u32,
    pub normal_effect_sound_id: Option<SoundId>,
    pub attack_sound_id: Option<SoundId>,
    pub hitted_sound_id: Option<SoundId>,
    pub hand_hit_effect_id: Option<EffectId>,
    pub die_effect_file_id: Option<EffectFileId>,
    pub die_sound_id: Option<SoundId>,
    pub npc_quest_type: u32,
    pub glow_colour: (f32, f32, f32),
    pub create_effect_index: u32,
    pub create_sound_id: Option<SoundId>,
    pub death_quest_trigger_name: String,
    pub npc_height: i32,
    pub motion_data: Vec<(MotionId, MotionFileData)>,
}

pub struct NpcConversationData {
    pub index: usize,
    pub name: String,
    pub _type: String,
    pub description: String,
    pub filename: String,
}

pub struct NpcStoreTabData {
    pub name: &'static str,
    pub items: HashMap<u16, ItemReference>,
}

pub struct NpcDatabaseOptions {
    pub load_frame_data: bool,
}

pub struct NpcDatabase {
    _string_database: Arc<StringDatabase>,
    npcs: Vec<Option<NpcData>>,
    conversation_files: HashMap<String, NpcConversationData>,
    store_tabs: HashMap<NpcStoreTabId, NpcStoreTabData>,
    action_map: EnumMap<NpcMotionAction, MotionId>,
}

impl NpcDatabase {
    pub fn new(
        string_database: Arc<StringDatabase>,
        npcs: Vec<Option<NpcData>>,
        conversation_files: HashMap<String, NpcConversationData>,
        store_tabs: HashMap<NpcStoreTabId, NpcStoreTabData>,
        action_map: EnumMap<NpcMotionAction, MotionId>,
    ) -> Self {
        Self {
            _string_database: string_database,
            npcs,
            conversation_files,
            store_tabs,
            action_map,
        }
    }

    pub fn get_npc(&self, id: NpcId) -> Option<&NpcData> {
        match self.npcs.get(id.get() as usize) {
            Some(inner) => inner.as_ref(),
            None => None,
        }
    }

    pub fn get_conversation(&self, key: &NpcConversationId) -> Option<&NpcConversationData> {
        self.conversation_files.get(&key.0)
    }

    pub fn find_conversation(&self, index: usize) -> Option<&NpcConversationData> {
        self.conversation_files
            .iter()
            .find(|(_, conv)| conv.index == index)
            .map(|(_, conv)| conv)
    }

    pub fn get_npc_motion(&self, npc_id: NpcId, motion_id: MotionId) -> Option<&MotionFileData> {
        let npc_data = self.get_npc(npc_id)?;
        npc_data
            .motion_data
            .iter()
            .find(|(id, _)| *id == motion_id)
            .map(|(_, data)| data)
    }

    pub fn get_npc_action_motion(
        &self,
        npc_id: NpcId,
        action: NpcMotionAction,
    ) -> Option<&MotionFileData> {
        self.get_npc_motion(npc_id, self.action_map[action])
    }

    pub fn get_store_tab(&self, id: NpcStoreTabId) -> Option<&NpcStoreTabData> {
        self.store_tabs.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &NpcData> {
        self.npcs.iter().filter_map(|npc_data| npc_data.as_ref())
    }
}
