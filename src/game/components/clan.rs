use bevy::prelude::{Component, Deref, DerefMut, Entity};

use rose_data::{ClanMemberPosition, SkillId};
use rose_game_common::components::{ClanLevel, ClanMark, ClanPoints, ClanUniqueId, Level, Money};

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct ClanMembership(pub Option<Entity>);

#[derive(Clone)]
pub enum ClanMember {
    Online {
        entity: Entity,
        position: ClanMemberPosition,
        contribution: ClanPoints,
    },
    Offline {
        name: String,
        position: ClanMemberPosition,
        contribution: ClanPoints,
        level: Level,
        job: u16,
    },
}

impl ClanMember {
    pub fn position(&self) -> ClanMemberPosition {
        match self {
            ClanMember::Online { position, .. } => *position,
            ClanMember::Offline { position, .. } => *position,
        }
    }

    pub fn contribution(&self) -> ClanPoints {
        match self {
            ClanMember::Online { contribution, .. } => *contribution,
            ClanMember::Offline { contribution, .. } => *contribution,
        }
    }
}

#[derive(Component)]
pub struct Clan {
    pub unique_id: ClanUniqueId,
    pub name: String,
    pub description: String,
    pub money: Money,
    pub points: ClanPoints,
    pub level: ClanLevel,
    pub members: Vec<ClanMember>,
    pub mark: ClanMark,
    pub skills: Vec<SkillId>,
}

impl Clan {
    pub fn find_online_member(&self, entity: Entity) -> Option<&ClanMember> {
        self.members.iter().find(|member| match member {
            ClanMember::Online {
                entity: member_entity,
                ..
            } => *member_entity == entity,
            _ => false,
        })
    }

    pub fn find_online_member_mut(&mut self, entity: Entity) -> Option<&mut ClanMember> {
        self.members.iter_mut().find(|member| match member {
            ClanMember::Online {
                entity: member_entity,
                ..
            } => *member_entity == entity,
            _ => false,
        })
    }

    pub fn find_offline_member(&self, name: &str) -> Option<&ClanMember> {
        self.members.iter().find(|member| match member {
            ClanMember::Offline {
                name: member_name, ..
            } => member_name == name,
            _ => false,
        })
    }

    pub fn find_offline_member_mut(&mut self, name: &str) -> Option<&mut ClanMember> {
        self.members.iter_mut().find(|member| match member {
            ClanMember::Offline {
                name: member_name, ..
            } => member_name == name,
            _ => false,
        })
    }
}
