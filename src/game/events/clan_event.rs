use bevy::prelude::Entity;

use rose_game_common::components::ClanMark;

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
}
