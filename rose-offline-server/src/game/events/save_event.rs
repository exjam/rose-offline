use bevy::ecs::prelude::Entity;

pub enum SaveEvent {
    Character {
        entity: Entity,
        remove_after_save: bool,
    },
}
