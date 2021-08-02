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

macro_rules! id_wrapper_impl {
    ($name:ident, String) => {
        impl $name {
            pub fn new(value: String) -> Self {
                Self(value)
            }

            #[allow(dead_code)]
            pub fn get(&self) -> &str {
                &self.0
            }
        }
    };
    ($name:ident, $value_type:ty) => {
        impl $name {
            pub fn new(value: $value_type) -> Self {
                Self(value)
            }

            #[allow(dead_code)]
            pub fn get(&self) -> $value_type {
                self.0
            }
        }

        impl FromStr for $name {
            type Err = <$value_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name(s.parse::<$value_type>()?))
            }
        }
    };
    ($name:ident, $inner_type:ty, $value_type:ty) => {
        impl $name {
            pub fn new(value: $value_type) -> Option<Self> {
                <$inner_type>::new(value).map($name)
            }

            #[allow(dead_code)]
            pub fn get(&self) -> $value_type {
                self.0.get()
            }
        }

        impl FromStr for $name {
            type Err = <$inner_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name(s.parse::<$inner_type>()?))
            }
        }
    };
}

mod ability;
mod ai_database;
mod drop_table;
mod item_database;
mod motion_database;
mod npc_database;
mod quest_database;
mod skill_database;
mod status_effect_database;
mod world;
mod zone_database;

pub mod account;
pub mod character;
pub mod formats;
pub mod item;

pub use ability::{AbilityType, AbilityValueCalculator, Damage, GetAbilityValues};
pub use ai_database::AiDatabase;
pub use character::{CharacterCreator, CharacterCreatorError};
pub use drop_table::DropTable;
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemData, ItemDatabase, ItemGradeData, ItemReference,
    JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData, VehicleItemData,
    WeaponItemData,
};
pub use motion_database::{MotionCharacterAction, MotionDatabase, MotionFileData, MotionId};
pub use npc_database::{
    NpcConversationData, NpcConversationId, NpcData, NpcDatabase, NpcId, NpcMotionAction,
};
pub use quest_database::{QuestData, QuestDatabase, QuestTrigger, QuestTriggerHash};
pub use skill_database::{
    SkillActionMode, SkillAddAbility, SkillCooldown, SkillCooldownGroup, SkillData, SkillDatabase,
    SkillId, SkillPageType, SkillTargetFilter, SkillType,
};
pub use status_effect_database::{
    StatusEffectClearedByType, StatusEffectData, StatusEffectDatabase, StatusEffectId,
    StatusEffectType,
};
pub use world::{WorldTicks, WORLD_TICK_DURATION};
pub use zone_database::{ZoneData, ZoneDatabase, ZoneId, ZoneMonsterSpawnPoint, ZoneNpcSpawn};
