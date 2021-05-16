use crate::data::{CharacterCreator, ItemDatabase, NpcDatabase, SkillDatabase, ZoneDatabase};

pub struct GameData {
    pub character_creator: Box<dyn CharacterCreator + Send + Sync>,
    pub items: ItemDatabase,
    pub npcs: NpcDatabase,
    pub skills: SkillDatabase,
    pub zones: ZoneDatabase,
}
