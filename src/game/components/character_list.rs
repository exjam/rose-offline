use std::char;

use crate::game::data::character::CharacterStorage;

use super::Account;

pub struct CharacterList {
    pub characters: Vec<CharacterStorage>,
}

impl CharacterList {
    pub fn new() -> CharacterList {
        CharacterList {
            characters: Vec::new(),
        }
    }
}
