use bevy::ecs::prelude::Component;

use crate::game::storage::character::CharacterStorage;

#[derive(Component)]
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
