use bevy::prelude::Entity;

pub enum RevivePosition {
    CurrentZone,
    SaveZone,
}

pub struct ReviveEvent {
    pub entity: Entity,
    pub position: RevivePosition,
}
