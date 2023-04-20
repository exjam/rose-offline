use bevy::ecs::prelude::Entity;
use rose_game_common::messages::{PartyItemSharing, PartyRejectInviteReason, PartyXpSharing};

use crate::game::components::CharacterUniqueId;

#[derive(Clone)]
pub enum PartyMemberEvent {
    Reconnect {
        party_entity: Entity,
        reconnect_entity: Entity,
        character_id: CharacterUniqueId,
        name: String,
    },
    Disconnect {
        party_entity: Entity,
        disconnect_entity: Entity,
        character_id: CharacterUniqueId,
        name: String,
    },
}

#[derive(Clone)]
pub enum PartyEvent {
    Invite {
        owner_entity: Entity,
        invited_entity: Entity,
    },
    AcceptInvite {
        owner_entity: Entity,
        invited_entity: Entity,
    },
    RejectInvite {
        reason: PartyRejectInviteReason,
        owner_entity: Entity,
        invited_entity: Entity,
    },
    ChangeOwner {
        owner_entity: Entity,
        new_owner_entity: Entity,
    },
    Leave {
        leaver_entity: Entity,
    },
    Kick {
        owner_entity: Entity,
        kick_character_id: CharacterUniqueId,
    },
    UpdateRules {
        owner_entity: Entity,
        item_sharing: PartyItemSharing,
        xp_sharing: PartyXpSharing,
    },
}
