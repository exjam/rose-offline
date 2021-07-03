mod ai_database;
mod item_database;
mod motion_database;
mod npc_database;
mod skill_database;
mod zone_database;

pub mod account;
pub mod character;
pub mod formats;
pub mod item;

use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref LOCAL_STORAGE_DIR: PathBuf = {
        let project = ProjectDirs::from("", "", "rose-offline").unwrap();
        PathBuf::from(project.data_local_dir())
    };
    pub static ref ACCOUNT_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("accounts");
    pub static ref CHARACTER_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("characters");
}

use crate::game::components::{
    AbilityValues, BasicStats, CharacterInfo, Equipment, Inventory, Level, SkillList,
};

#[derive(Clone, Copy)]
pub struct Damage {
    pub amount: u32,
    pub is_critical: bool,
    pub apply_hit_stun: bool,
}

pub trait AbilityValueCalculator {
    fn calculate(
        &self,
        character_info: &CharacterInfo,
        level: &Level,
        equipment: &Equipment,
        inventory: &Inventory,
        basic_stats: &BasicStats,
        skill_list: &SkillList,
    ) -> AbilityValues;

    fn calculate_npc(&self, npc_id: usize) -> Option<AbilityValues>;

    fn calculate_damage(
        &self,
        attacker: &AbilityValues,
        defender: &AbilityValues,
        hit_count: i32,
    ) -> Damage;

    fn calculate_give_xp(
        &self,
        attacker_level: i32,
        attacker_damage: i32,
        defender_level: i32,
        defender_max_hp: i32,
        defender_reward_xp: i32,
        world_xp_rate: i32,
    ) -> i32;
}

pub use ai_database::AiDatabase;
pub use character::{CharacterCreator, CharacterCreatorError};
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemData, ItemDatabase, ItemGradeData, ItemReference,
    JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData, VehicleItemData,
    WeaponItemData,
};
pub use motion_database::{MotionCharacterAction, MotionDatabase, MotionFileData};
pub use npc_database::{
    NpcConversationData, NpcConversationReference, NpcData, NpcDatabase, NpcMotionAction,
    NpcReference,
};
pub use skill_database::{
    SkillAddAbility, SkillData, SkillDatabase, SkillPage, SkillReference, SkillType,
};
pub use zone_database::{ZoneData, ZoneDatabase, ZoneMonsterSpawnPoint, ZoneNpcSpawn};
