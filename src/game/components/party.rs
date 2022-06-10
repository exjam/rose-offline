use arrayvec::ArrayVec;
use bevy::ecs::prelude::{Component, Entity};
use rose_game_common::messages::{PartyItemSharing, PartyXpSharing};

use crate::game::components::CharacterUniqueId;

#[derive(Clone)]
pub enum PartyMember {
    Online(Entity),
    Offline(CharacterUniqueId, String),
}

impl PartyMember {
    pub fn get_entity(&self) -> Option<Entity> {
        match self {
            PartyMember::Online(entity) => Some(*entity),
            PartyMember::Offline(_, _) => None,
        }
    }
}

#[derive(Component)]
pub struct Party {
    pub owner: Entity,
    pub members: ArrayVec<PartyMember, 5>,
    pub item_sharing: PartyItemSharing,
    pub xp_sharing: PartyXpSharing,
    pub average_member_level: i32,
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
            item_sharing: PartyItemSharing::EqualLootDistribution,
            xp_sharing: PartyXpSharing::EqualShare,
            average_member_level: 1,
            level: 1,
        }
    }
}
