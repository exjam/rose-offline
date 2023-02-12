use bevy::prelude::{Component, Deref, DerefMut};

use crate::game::storage::character::CharacterStorage;

#[derive(Component, Default, Deref, DerefMut)]
pub struct CharacterList {
    pub characters: Vec<CharacterStorage>,
}
