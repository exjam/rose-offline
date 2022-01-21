use bevy_ecs::prelude::Entity;

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
pub enum PartyEvent {
    MemberReconnect(PartyMemberReconnect),
    MemberDisconnect(PartyMemberDisconnect),
    Invite(PartyEventInvite),
    AcceptInvite(PartyEventInvite),
    RejectInvite(PartyEventInvite),
    ChangeOwner(PartyEventChangeOwner),
    Leave(PartyEventLeave),
    Kick(PartyEventKick),
}
