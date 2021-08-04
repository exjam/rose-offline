use bevy_ecs::prelude::Entity;

pub struct ChatCommandEvent {
    pub entity: Entity,
    pub command: String,
}

impl ChatCommandEvent {
    pub fn new(entity: Entity, command: String) -> Self {
        Self { entity, command }
    }
}
