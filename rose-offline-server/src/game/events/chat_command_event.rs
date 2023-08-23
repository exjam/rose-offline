use bevy::{ecs::prelude::Entity, prelude::Event};

#[derive(Event)]
pub struct ChatCommandEvent {
    pub entity: Entity,
    pub command: String,
}

impl ChatCommandEvent {
    pub fn new(entity: Entity, command: String) -> Self {
        Self { entity, command }
    }
}
