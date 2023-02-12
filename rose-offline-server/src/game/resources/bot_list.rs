use bevy::{
    ecs::prelude::Entity,
    prelude::{Deref, DerefMut, Resource},
};

pub struct BotListEntry {
    pub entity: Entity,
}

impl BotListEntry {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct BotList(Vec<BotListEntry>);

impl BotList {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}
