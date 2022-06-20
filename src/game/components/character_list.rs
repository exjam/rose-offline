use bevy::prelude::{Component, Deref, DerefMut};

use crate::game::storage::character::CharacterStorage;

#[derive(Component, Deref, DerefMut)]
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
