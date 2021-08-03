use bevy_ecs::prelude::Entity;

pub struct PendingCharacterSave {
    pub entity: Entity,
    pub remove_after_save: bool,
}

pub enum PendingSave {
    Character(PendingCharacterSave),
}

pub type PendingSaveList = Vec<PendingSave>;

impl PendingSave {
    pub fn with_character(entity: Entity, remove_after_save: bool) -> Self {
        Self::Character(PendingCharacterSave {
            entity,
            remove_after_save,
        })
    }
}
