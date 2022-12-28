use bevy::prelude::Entity;

use rose_data::SkillId;
use rose_game_common::components::{ClanLevel, ClanMark, ClanPoints, Money};

use crate::game::components::Level;

pub enum ClanEvent {
    Create {
        creator: Entity,
        name: String,
        description: String,
        mark: ClanMark,
    },
    MemberDisconnect {
        clan_entity: Entity,
        disconnect_entity: Entity,
        name: String,
        level: Level,
        job: u16,
    },
    GetMemberList {
        entity: Entity,
    },
    AddLevel {
        clan_entity: Entity,
        level: i32,
    },
    SetLevel {
        clan_entity: Entity,
        level: ClanLevel,
    },
    AddMoney {
        clan_entity: Entity,
        money: i64,
    },
    SetMoney {
        clan_entity: Entity,
        money: Money,
    },
    AddPoints {
        clan_entity: Entity,
        points: i64,
    },
    SetPoints {
        clan_entity: Entity,
        points: ClanPoints,
    },
    AddSkill {
        clan_entity: Entity,
        skill_id: SkillId,
    },
    RemoveSkill {
        clan_entity: Entity,
        skill_id: SkillId,
    },
}
