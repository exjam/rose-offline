pub mod account;
pub mod character;
pub mod formats;
pub mod item;
mod item_database;
mod npc_database;
mod skill_database;
mod zone_database;

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
}

pub use character::{CharacterCreator, CharacterCreatorError};
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemData, ItemDatabase, ItemGradeData, ItemReference,
    JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData, VehicleItemData,
    WeaponItemData,
};
pub use npc_database::{
    NpcConversationData, NpcConversationReference, NpcData, NpcDatabase, NpcReference,
};
pub use skill_database::{
    SkillAddAbility, SkillData, SkillDatabase, SkillPage, SkillReference, SkillType,
};
pub use zone_database::{ZoneData, ZoneDatabase, ZoneMonsterSpawnPoint, ZoneNpcSpawn};
