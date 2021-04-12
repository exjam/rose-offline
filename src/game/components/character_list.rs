use std::char;

use crate::game::data::character::CharacterStorage;

use super::{CharacterDeleteTime, CharacterInfo, Equipment, Level};

#[derive(Clone)]
pub struct CharacterListItem {
    pub info: CharacterInfo,
    pub level: Level,
    pub delete_time: Option<CharacterDeleteTime>,
    pub equipment: Equipment,
}

impl From<CharacterStorage> for CharacterListItem {
    fn from(storage: CharacterStorage) -> CharacterListItem {
        CharacterListItem {
            info: storage.info,
            delete_time: storage.delete_time,
            equipment: storage.equipment,
            level: storage.level,
        }
    }
}

#[derive(Clone)]
pub struct CharacterList {
    pub characters: Vec<CharacterListItem>,
}

impl CharacterList {
    pub fn new() -> CharacterList {
        CharacterList {
            characters: Vec::new(),
        }
    }
}
