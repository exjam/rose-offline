use bevy::prelude::{Entity, Event};

pub enum RevivePosition {
    CurrentZone,
    SaveZone,
}

#[derive(Event)]
pub struct ReviveEvent {
    pub entity: Entity,
    pub position: RevivePosition,
}
