use arrayvec::ArrayVec;
use bevy::ecs::prelude::{Component, Entity};

use crate::game::components::CharacterUniqueId;

#[derive(Clone)]
pub enum PartyMember {
    Online(Entity),
    Offline(CharacterUniqueId, String),
}

#[derive(Component)]
pub struct Party {
    pub owner: Entity,
    pub members: ArrayVec<PartyMember, 5>,
    pub level: i32,
}

impl Party {
    pub fn new(owner: Entity, party_members: &[PartyMember]) -> Self {
        let mut members = ArrayVec::new();
        for member in party_members {
            members.push(member.clone());
        }
        Self {
            owner,
            members,
            level: 1,
        }
    }
}
