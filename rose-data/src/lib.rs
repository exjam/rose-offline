macro_rules! id_wrapper_impl {
    ($name:ident, String) => {
        impl $name {
            #[allow(dead_code)]
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
            #[allow(dead_code)]
            pub fn new(value: $value_type) -> Self {
                Self(value)
            }

            #[allow(dead_code)]
            pub fn get(&self) -> $value_type {
                self.0
            }
        }

        #[allow(dead_code)]
        impl FromStr for $name {
            type Err = <$value_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name(s.parse::<$value_type>()?))
            }
        }
    };
    ($name:ident, $inner_type:ty, $value_type:ty) => {
        impl $name {
            #[allow(dead_code)]
            pub fn new(value: $value_type) -> Option<Self> {
                <$inner_type>::new(value).map($name)
            }

            #[allow(dead_code)]
            pub fn get(&self) -> $value_type {
                self.0.get()
            }
        }

        #[allow(dead_code)]
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
mod animation_event_flags;
mod character_motion_database;
mod data_decoder;
mod effect_database;
mod item;
mod item_database;
mod motion_file_data;
mod npc_database;
mod quest_database;
mod skill_database;
mod skybox_database;
mod sound_database;
mod status_effect_database;
mod string_database;
mod warp_gate_database;
mod world;
mod zone_database;
mod zone_list;

pub use ability::AbilityType;
pub use ai_database::AiDatabase;
pub use animation_event_flags::AnimationEventFlags;
pub use character_motion_database::{
    CharacterMotionAction, CharacterMotionDatabase, CharacterMotionDatabaseOptions,
};
pub use data_decoder::DataDecoder;
pub use effect_database::{
    EffectBulletMoveType, EffectData, EffectDatabase, EffectFileId, EffectId,
};
pub use item::{
    AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemSlotBehaviour, ItemWeaponType, StackError,
    StackableItem, StackableSlotBehaviour, VehiclePartIndex,
};
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemClass, ItemData, ItemDatabase, ItemGradeData,
    ItemReference, ItemType, JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData,
    VehicleItemData, VehicleItemPart, WeaponItemData,
};
pub use motion_file_data::{MotionFileData, MotionId};
pub use npc_database::{
    NpcConversationData, NpcConversationId, NpcData, NpcDatabase, NpcDatabaseOptions, NpcId,
    NpcMotionAction, NpcStoreTabData, NpcStoreTabId,
};
pub use quest_database::{QuestData, QuestDatabase, QuestTrigger, QuestTriggerHash};
pub use skill_database::{
    SkillActionMode, SkillAddAbility, SkillBasicCommand, SkillCastingEffect, SkillCooldown,
    SkillCooldownGroup, SkillData, SkillDatabase, SkillId, SkillPageType, SkillTargetFilter,
    SkillType,
};
pub use skybox_database::{SkyboxData, SkyboxDatabase, SkyboxId, SkyboxState};
pub use sound_database::{SoundData, SoundDatabase, SoundId};
pub use status_effect_database::{
    StatusEffectClearedByType, StatusEffectData, StatusEffectDatabase, StatusEffectId,
    StatusEffectType,
};
pub use string_database::StringDatabase;
pub use warp_gate_database::{WarpGateData, WarpGateDatabase, WarpGateId};
pub use world::{
    WorldTicks, WORLD_DAYS_PER_MONTH, WORLD_MONTH_PER_YEAR, WORLD_TICKS_PER_DAY,
    WORLD_TICKS_PER_MONTH, WORLD_TICKS_PER_YEAR, WORLD_TICK_DURATION,
};
pub use zone_database::{
    ZoneData, ZoneDatabase, ZoneEventObject, ZoneId, ZoneMonsterSpawnPoint, ZoneNpcSpawn,
};
pub use zone_list::{ZoneList, ZoneListEntry};
