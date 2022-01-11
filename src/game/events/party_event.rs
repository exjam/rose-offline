use bevy_ecs::prelude::Entity;

use crate::game::components::CharacterUniqueId;

pub struct PartyEventInvite {
    pub owner_entity: Entity,
    pub invited_entity: Entity,
}

pub struct PartyEventLeave {
    pub leaver_entity: Entity,
}

pub struct PartyEventChangeOwner {
    pub owner_entity: Entity,
    pub new_owner_entity: Entity,
}

pub struct PartyEventKick {
    pub owner_entity: Entity,
    pub kick_character_id: CharacterUniqueId,
}

pub enum PartyEvent {
    Invite(PartyEventInvite),
    AcceptInvite(PartyEventInvite),
    RejectInvite(PartyEventInvite),
    ChangeOwner(PartyEventChangeOwner),
    Leave(PartyEventLeave),
    Kick(PartyEventKick),
}
