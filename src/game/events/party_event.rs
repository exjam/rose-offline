use bevy::ecs::prelude::Entity;
use rose_game_common::messages::{PartyItemSharing, PartyRejectInviteReason, PartyXpSharing};

use crate::game::components::CharacterUniqueId;

#[derive(Clone)]
pub struct PartyMemberReconnect {
    pub party_entity: Entity,
    pub reconnect_entity: Entity,
    pub character_id: CharacterUniqueId,
    pub name: String,
}

#[derive(Clone)]
pub struct PartyMemberDisconnect {
    pub party_entity: Entity,
    pub disconnect_entity: Entity,
    pub character_id: CharacterUniqueId,
    pub name: String,
}

#[derive(Clone)]
pub struct PartyEventInvite {
    pub owner_entity: Entity,
    pub invited_entity: Entity,
}

#[derive(Clone)]
pub struct PartyEventLeave {
    pub leaver_entity: Entity,
}

#[derive(Clone)]
pub struct PartyEventChangeOwner {
    pub owner_entity: Entity,
    pub new_owner_entity: Entity,
}

#[derive(Clone)]
pub struct PartyEventKick {
    pub owner_entity: Entity,
    pub kick_character_id: CharacterUniqueId,
}

#[derive(Clone)]
pub struct PartyEventUpdateRules {
    pub owner_entity: Entity,
    pub item_sharing: PartyItemSharing,
    pub xp_sharing: PartyXpSharing,
}

#[derive(Clone)]
pub enum PartyMemberEvent {
    Reconnect(PartyMemberReconnect),
    Disconnect(PartyMemberDisconnect),
}

#[derive(Clone)]
pub enum PartyEvent {
    Invite(PartyEventInvite),
    AcceptInvite(PartyEventInvite),
    RejectInvite(PartyRejectInviteReason, PartyEventInvite),
    ChangeOwner(PartyEventChangeOwner),
    Leave(PartyEventLeave),
    Kick(PartyEventKick),
    UpdateRules(PartyEventUpdateRules),
}
