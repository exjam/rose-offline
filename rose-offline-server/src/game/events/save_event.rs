use bevy::{ecs::prelude::Entity, prelude::Event};

#[derive(Event)]
pub enum SaveEvent {
    Character {
        entity: Entity,
        remove_after_save: bool,
    },
}
