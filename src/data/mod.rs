mod ai_database;
mod drop_table;
mod item_database;
mod motion_database;
mod npc_database;
mod quest_database;
mod skill_database;
mod zone_database;

pub mod ability;
pub mod account;
pub mod character;
pub mod formats;
pub mod item;

use directories::ProjectDirs;
use lazy_static::lazy_static;
use num_derive::NumOps;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr, time::Duration};

lazy_static! {
    pub static ref LOCAL_STORAGE_DIR: PathBuf = {
        let project = ProjectDirs::from("", "", "rose-offline").unwrap();
        PathBuf::from(project.data_local_dir())
    };
    pub static ref ACCOUNT_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("accounts");
    pub static ref CHARACTER_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("characters");
}

use crate::game::components::{
    AbilityValues, BasicStatType, BasicStats, CharacterInfo, Equipment, Inventory, Level, SkillList,
};

pub const WORLD_TICK_DURATION: Duration = Duration::from_secs(10);

#[derive(Copy, Clone, Debug, NumOps, Deserialize, Serialize)]
pub struct WorldTicks(pub u64);

impl From<WorldTicks> for Duration {
    fn from(ticks: WorldTicks) -> Duration {
        Duration::from_millis(ticks.0 * WORLD_TICK_DURATION.as_millis() as u64)
    }
}

impl FromStr for WorldTicks {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u64>().map_err(|_| ())?;
        Ok(WorldTicks(value))
    }
}

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

    fn calculate_give_stamina(
        &self,
        experience_points: i32,
        level: i32,
        world_stamina_rate: i32,
    ) -> i32;

    fn calculate_basic_stat_increase_cost(
        &self,
        basic_stats: &BasicStats,
        basic_stat_type: BasicStatType,
    ) -> Option<u32>;

    fn calculate_levelup_require_xp(&self, level: u32) -> u64;
    fn calculate_levelup_reward_skill_points(&self, level: u32) -> u32;
    fn calculate_levelup_reward_stat_points(&self, level: u32) -> u32;

    fn calculate_reward_value(
        &self,
        equation_id: usize,
        base_reward_value: i32,
        dup_count: i32,
        level: i32,
        charm: i32,
        fame: i32,
        world_reward_rate: i32,
    ) -> i32;
}

pub use ai_database::AiDatabase;
pub use character::{CharacterCreator, CharacterCreatorError};
pub use drop_table::DropTable;
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemData, ItemDatabase, ItemGradeData, ItemReference,
    JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData, VehicleItemData,
    WeaponItemData,
};
pub use motion_database::{MotionCharacterAction, MotionDatabase, MotionFileData, MotionReference};
pub use npc_database::{
    NpcConversationData, NpcConversationReference, NpcData, NpcDatabase, NpcMotionAction,
    NpcReference,
};
pub use quest_database::{QuestData, QuestDatabase, QuestTrigger, QuestTriggerHash};
pub use skill_database::{
    SkillAddAbility, SkillData, SkillDatabase, SkillPageType, SkillReference, SkillType,
};
pub use zone_database::{
    ZoneData, ZoneDatabase, ZoneMonsterSpawnPoint, ZoneNpcSpawn, ZoneReference,
};
