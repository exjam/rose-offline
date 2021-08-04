use bevy_ecs::prelude::Entity;

pub struct SaveEventCharacter {
    pub entity: Entity,
    pub remove_after_save: bool,
}

pub enum SaveEvent {
    Character(SaveEventCharacter),
}

impl SaveEvent {
    pub fn with_character(entity: Entity, remove_after_save: bool) -> Self {
        Self::Character(SaveEventCharacter {
            entity,
            remove_after_save,
        })
    }
}
