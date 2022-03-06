use std::sync::Arc;

use crate::{
    data::{
        AbilityValueCalculator, AiDatabase, DataDecoder, DropTable, ItemDatabase, MotionDatabase,
        NpcDatabase, QuestDatabase, SkillDatabase, StatusEffectDatabase, WarpGateDatabase,
        ZoneDatabase,
    },
    game::storage::character::CharacterCreator,
};

pub struct GameData {
    pub character_creator: Box<dyn CharacterCreator + Send + Sync>,
    pub ability_value_calculator: Box<dyn AbilityValueCalculator + Send + Sync>,
    pub data_decoder: Box<dyn DataDecoder + Send + Sync>,
    pub drop_table: Box<dyn DropTable + Send + Sync>,
    pub ai: Arc<AiDatabase>,
    pub items: Arc<ItemDatabase>,
    pub motions: Arc<MotionDatabase>,
    pub npcs: Arc<NpcDatabase>,
    pub quests: Arc<QuestDatabase>,
    pub skills: Arc<SkillDatabase>,
    pub status_effects: Arc<StatusEffectDatabase>,
    pub warp_gates: Arc<WarpGateDatabase>,
    pub zones: Arc<ZoneDatabase>,
}
