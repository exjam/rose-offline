use std::sync::Arc;

use crate::data::{
    AbilityValueCalculator, CharacterCreator, ItemDatabase, NpcDatabase, SkillDatabase,
    ZoneDatabase,
};

pub struct GameData {
    pub character_creator: Box<dyn CharacterCreator + Send + Sync>,
    pub ability_value_calculator: Box<dyn AbilityValueCalculator + Send + Sync>,
    pub items: Arc<ItemDatabase>,
    pub npcs: Arc<NpcDatabase>,
    pub skills: Arc<SkillDatabase>,
    pub zones: Arc<ZoneDatabase>,
}
