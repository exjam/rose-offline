use std::sync::Arc;

use crate::data::{
    AbilityValueCalculator, CharacterCreator, ItemDatabase, MotionDatabase, NpcDatabase,
    SkillDatabase, ZoneDatabase,
};

pub struct GameData {
    pub character_creator: Box<dyn CharacterCreator + Send + Sync>,
    pub ability_value_calculator: Box<dyn AbilityValueCalculator + Send + Sync>,
    pub items: Arc<ItemDatabase>,
    pub motions: Arc<MotionDatabase>,
    pub npcs: Arc<NpcDatabase>,
    pub skills: Arc<SkillDatabase>,
    pub zones: Arc<ZoneDatabase>,
}
