use bevy::ecs::{
    prelude::{Changed, Commands, Entity, EventReader, Or, Query},
    query::WorldQuery,
};
use rose_game_common::{
    components::Level,
    messages::{PartyItemSharing, PartyRejectInviteReason, PartyXpSharing},
};

use crate::game::{
    components::{
        AbilityValues, BotAi, BotMessage, CharacterInfo, CharacterUniqueId, ClientEntity,
        GameClient, HealthPoints, Party, PartyMember, PartyMembership, Stamina, StatusEffects,
    },
    events::{
        PartyEvent, PartyEventChangeOwner, PartyEventInvite, PartyEventKick, PartyEventLeave,
        PartyEventUpdateRules, PartyMemberDisconnect, PartyMemberEvent, PartyMemberReconnect,
    },
    messages::server::{
        PartyMemberInfo, PartyMemberInfoOffline, PartyMemberInfoOnline, PartyMemberLeave,
        PartyMemberList, ServerMessage,
    },
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct PartyMembershipQuery<'w> {
    client_entity: &'w ClientEntity,
    character_info: &'w CharacterInfo,
    party_membership: &'w mut PartyMembership,
    game_client: Option<&'w GameClient>,
    bot_ai: Option<&'w mut BotAi>,
}

#[derive(WorldQuery)]
pub struct PartyMemberInfoQuery<'w> {
    ability_values: &'w AbilityValues,
    character_info: &'w CharacterInfo,
    client_entity: &'w ClientEntity,
    health_points: &'w HealthPoints,
    status_effects: &'w StatusEffects,
    stamina: &'w Stamina,
    game_client: Option<&'w GameClient>,
}

/*
Party event system TODO:
- Check party level / team requirements?
- Party XP / drops
*/

fn send_message_to_members(
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    party_members: &[PartyMember],
    message: ServerMessage,
    except_entity: Option<Entity>,
) {
    for party_member in party_members.iter() {
        if let PartyMember::Online(party_member_entity) = party_member {
            if let Some(except_entity) = except_entity {
                if *party_member_entity == except_entity {
                    continue;
                }
            }

            if let Some(game_client) = party_member_info_query
                .get(*party_member_entity)
                .ok()
                .and_then(|info| info.game_client)
            {
                game_client.server_message_tx.send(message.clone()).ok();
            }
        }
    }
}

fn delete_party(
    commands: &mut Commands,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_entity: Entity,
    party: &mut Party,
) {
    for party_member in party.members.iter() {
        if let PartyMember::Online(party_member_entity) = party_member {
            if let Ok(mut party_membership) = party_membership_query.get_mut(*party_member_entity) {
                if let Some(member_game_client) = party_membership.game_client {
                    member_game_client
                        .server_message_tx
                        .send(ServerMessage::PartyDelete)
                        .ok();
                }

                *party_membership.party_membership = PartyMembership::None;
            }
        }
    }
    party.members.clear();
    commands.entity(party_entity).despawn();
}

enum PartyInviteError {
    AlreadyHasParty,
    NoPermission,
    PartyFull,
    InvalidEntity,
}

fn handle_party_invite(
    party_membership_query: &mut Query<PartyMembershipQuery>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let [owner, invited] = party_membership_query
        .get_many_mut([owner_entity, invited_entity])
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    if !matches!(*invited.party_membership, PartyMembership::None) {
        return Err(PartyInviteError::AlreadyHasParty);
    }

    if let Some(invited_game_client) = invited.game_client {
        let message = if matches!(*owner.party_membership, PartyMembership::None) {
            ServerMessage::PartyCreate(owner.client_entity.id)
        } else {
            ServerMessage::PartyInvite(owner.client_entity.id)
        };

        invited_game_client.server_message_tx.send(message).ok();
    }

    if let Some(mut invited_bot_ai) = invited.bot_ai {
        invited_bot_ai
            .messages
            .push(BotMessage::PartyInvite(owner_entity));
    }

    Ok(())
}

fn get_online_party_member_info(
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    entity: Entity,
) -> Option<PartyMemberInfoOnline> {
    let party_member = party_member_info_query.get(entity).ok()?;

    Some(PartyMemberInfoOnline {
        character_id: party_member.character_info.unique_id,
        name: party_member.character_info.name.clone(),
        entity_id: party_member.client_entity.id,
        health_points: *party_member.health_points,
        status_effects: party_member.status_effects.active.clone(),
        max_health: party_member.ability_values.get_max_health(),
        concentration: party_member.ability_values.get_concentration(),
        health_recovery: party_member.ability_values.get_additional_health_recovery(), // TODO: ??
        mana_recovery: party_member.ability_values.get_additional_mana_recovery(),     // TODO: ??
        stamina: *party_member.stamina,
    })
}

fn get_party_membership_info(
    party_members: &[PartyMember],
    party_member_info_query: &Query<PartyMemberInfoQuery>,
) -> Vec<PartyMemberInfo> {
    let mut info = Vec::new();
    for party_member in party_members.iter() {
        match party_member {
            &PartyMember::Online(party_member_entity) => {
                if let Some(online_party_member_info) =
                    get_online_party_member_info(party_member_info_query, party_member_entity)
                {
                    info.push(PartyMemberInfo::Online(online_party_member_info));
                }
            }
            PartyMember::Offline(character_id, name) => {
                info.push(PartyMemberInfo::Offline(PartyMemberInfoOffline {
                    character_id: *character_id,
                    name: name.clone(),
                }));
            }
        }
    }
    info
}

fn handle_party_accept_invite(
    commands: &mut Commands,
    party_query: &mut Query<&mut Party>,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let [mut owner, mut invited] = party_membership_query
        .get_many_mut([owner_entity, invited_entity])
        .map_err(|_| PartyInviteError::InvalidEntity)?;

    // First ensure invited entity is not already in a party
    if !matches!(*invited.party_membership, PartyMembership::None) {
        return Err(PartyInviteError::AlreadyHasParty);
    }

    let is_create_party = matches!(*owner.party_membership, PartyMembership::None);
    let (item_sharing, xp_sharing, party_members) = match *owner.party_membership {
        PartyMembership::None => {
            // Create a new party
            let party = Party::new(
                owner_entity,
                &[
                    PartyMember::Online(owner_entity),
                    PartyMember::Online(invited_entity),
                ],
            );
            let item_sharing = party.item_sharing;
            let xp_sharing = party.xp_sharing;
            let party_members = party.members.clone();
            let party_entity = commands.spawn(party).id();

            *owner.party_membership = PartyMembership::new(party_entity);
            *invited.party_membership = PartyMembership::new(party_entity);

            (item_sharing, xp_sharing, party_members)
        }
        PartyMembership::Member(party_entity) => {
            // Add to current party
            let mut party = party_query
                .get_mut(party_entity)
                .expect("PartyMembership pointing to invalid party entity");

            if owner_entity != party.owner {
                return Err(PartyInviteError::NoPermission);
            }

            if party.members.len() >= party.members.capacity() {
                return Err(PartyInviteError::PartyFull);
            }

            party.members.push(PartyMember::Online(invited_entity));
            *invited.party_membership = PartyMembership::new(party_entity);

            (party.item_sharing, party.xp_sharing, party.members.clone())
        }
    };

    // Send accept create to owner
    if is_create_party {
        if let Some(owner_game_client) = owner.game_client {
            owner_game_client
                .server_message_tx
                .send(ServerMessage::PartyAcceptCreate(invited.client_entity.id))
                .ok();
        }
    }

    let party_member_infos = get_party_membership_info(&party_members, party_member_info_query);
    let (invited_member_info, other_members_info): (Vec<_>, Vec<_>) =
        party_member_infos.into_iter().partition(|member_info| {
            if let PartyMemberInfo::Online(online_party_member) = member_info {
                online_party_member.entity_id == invited.client_entity.id
            } else {
                false
            }
        });

    // Send list of other members to invited
    if let Some(invited_game_client) = invited.game_client {
        invited_game_client
            .server_message_tx
            .send(ServerMessage::PartyMemberList(PartyMemberList {
                item_sharing,
                xp_sharing,
                owner_character_id: owner.character_info.unique_id,
                members: other_members_info,
            }))
            .ok();
    }

    // Send info about invited to other members
    send_message_to_members(
        party_member_info_query,
        &party_members,
        ServerMessage::PartyMemberList(PartyMemberList {
            item_sharing,
            xp_sharing,
            owner_character_id: owner.character_info.unique_id,
            members: invited_member_info,
        }),
        Some(invited_entity),
    );

    Ok(())
}

fn handle_party_reject_invite(
    party_membership_query: &mut Query<PartyMembershipQuery>,
    reason: PartyRejectInviteReason,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let [owner, invited] = party_membership_query
        .get_many_mut([owner_entity, invited_entity])
        .map_err(|_| PartyInviteError::InvalidEntity)?;

    if let Some(owner_game_client) = owner.game_client {
        owner_game_client
            .server_message_tx
            .send(ServerMessage::PartyRejectInvite(
                reason,
                invited.client_entity.id,
            ))
            .ok();
    }

    Ok(())
}

enum PartyLeaveError {
    InvalidEntity,
    NotInParty,
}

fn handle_party_leave(
    commands: &mut Commands,
    party_query: &mut Query<&mut Party>,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    leaver_entity: Entity,
) -> Result<(), PartyLeaveError> {
    let mut leaver = party_membership_query
        .get_mut(leaver_entity)
        .map_err(|_| PartyLeaveError::InvalidEntity)?;

    let party_entity = if let PartyMembership::Member(party_entity) = *leaver.party_membership {
        party_entity
    } else {
        return Err(PartyLeaveError::NotInParty);
    };

    if let Some(leaver_game_client) = leaver.game_client {
        // Send party delete message to leaver
        leaver_game_client
            .server_message_tx
            .send(ServerMessage::PartyDelete)
            .ok();
    }

    *leaver.party_membership = PartyMembership::None;

    // Remove leaver from party
    let mut party = party_query
        .get_mut(party_entity)
        .expect("PartyMembership pointing to invalid party entity");
    party.members.retain(|party_member| match party_member {
        PartyMember::Online(party_member_entity) => *party_member_entity != leaver_entity,
        _ => true,
    });

    if party.owner == leaver_entity {
        // If owner is leaving, choose first online member to be new owner
        if let Some(new_owner_entity) =
            party
                .members
                .iter()
                .find_map(|party_member| match party_member {
                    PartyMember::Online(party_member_entity) => Some(*party_member_entity),
                    _ => None,
                })
        {
            party.owner = new_owner_entity;
        } else {
            // There are no other online party members, so cause the party to be deleted below
            party.members.clear();
        }
    }

    if party.members.len() <= 1 {
        delete_party(commands, party_membership_query, party_entity, &mut party);
    } else {
        // Get leaver character id and owner character id for leave message
        let [leaver, owner] = party_member_info_query
            .get_many([leaver_entity, party.owner])
            .unwrap();

        // Send message to other members informing of leaver and new owner
        send_message_to_members(
            party_member_info_query,
            &party.members,
            ServerMessage::PartyMemberLeave(PartyMemberLeave {
                leaver_character_id: leaver.character_info.unique_id,
                owner_character_id: owner.character_info.unique_id,
            }),
            None,
        );
    }

    Ok(())
}

enum PartyKickError {
    InvalidEntity,
    NotOwner,
    NotInParty,
    InvalidKickCharacter,
}

fn handle_party_kick(
    commands: &mut Commands,
    party_query: &mut Query<&mut Party>,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    owner_entity: Entity,
    kick_character_id: CharacterUniqueId,
) -> Result<(), PartyKickError> {
    // First ensure owner_entity is actually owner of the party
    let owner = party_membership_query
        .get_mut(owner_entity)
        .map_err(|_| PartyKickError::InvalidEntity)?;
    let party_entity = match *owner.party_membership {
        PartyMembership::None => return Err(PartyKickError::NotInParty),
        PartyMembership::Member(party_entity) => party_entity,
    };

    let mut party = party_query
        .get_mut(party_entity)
        .expect("PartyMembership pointing to invalid party entity");

    // Only party owner can kick members
    if party.owner != owner_entity {
        return Err(PartyKickError::NotOwner);
    }

    // Ensure we are not kicking ourself
    let owner = party_member_info_query
        .get(owner_entity)
        .map_err(|_| PartyKickError::InvalidEntity)?;
    if owner.character_info.unique_id == kick_character_id {
        return Err(PartyKickError::InvalidKickCharacter);
    }

    // Try to remove kicked member from party
    let mut kicked_online_entity = None;
    let mut kicked_offline = false;
    party.members.retain(|party_member| match *party_member {
        PartyMember::Online(party_member_entity) => party_member_info_query
            .get(party_member_entity)
            .map_or(true, |party_member| {
                if party_member.character_info.unique_id == kick_character_id {
                    kicked_online_entity = Some(party_member_entity);
                    false
                } else {
                    true
                }
            }),
        PartyMember::Offline(party_member_character_id, _) => {
            if party_member_character_id == kick_character_id {
                kicked_offline = true;
                false
            } else {
                true
            }
        }
    });

    if kicked_online_entity.is_none() && !kicked_offline {
        return Err(PartyKickError::InvalidKickCharacter);
    }

    // If the kicked character was online, update party membership
    if let Some(kicked_entity) = kicked_online_entity {
        let mut kicked = party_membership_query.get_mut(kicked_entity).unwrap();

        *kicked.party_membership = PartyMembership::None;

        if let Some(kicked_game_client) = kicked.game_client {
            kicked_game_client
                .server_message_tx
                .send(ServerMessage::PartyMemberKicked(kick_character_id))
                .ok();
        }
    }

    // Send kick message to other party member
    send_message_to_members(
        party_member_info_query,
        &party.members,
        ServerMessage::PartyMemberKicked(kick_character_id),
        None,
    );

    // If party is down to 1 member, delete the party
    if party.members.len() <= 1 {
        delete_party(commands, party_membership_query, party_entity, &mut party);
    }

    Ok(())
}

enum PartyChangeOwnerError {
    InvalidEntity,
    NotOwner,
    NotInParty,
    InvalidNewOwnerEntity,
}

fn handle_party_change_owner(
    party_query: &mut Query<&mut Party>,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    owner_entity: Entity,
    new_owner_entity: Entity,
) -> Result<(), PartyChangeOwnerError> {
    // Get owner entity's party
    let [owner, new_owner] = party_membership_query
        .get_many([owner_entity, new_owner_entity])
        .map_err(|_| PartyChangeOwnerError::InvalidEntity)?;

    // Ensure owner and new owner are in the same party
    let party_entity = match *owner.party_membership {
        PartyMembership::None => return Err(PartyChangeOwnerError::NotInParty),
        PartyMembership::Member(party_entity) => party_entity,
    };
    let new_owner_party_entity = match *new_owner.party_membership {
        PartyMembership::None => return Err(PartyChangeOwnerError::InvalidNewOwnerEntity),
        PartyMembership::Member(new_owner_party_entity) => new_owner_party_entity,
    };
    if new_owner_party_entity != party_entity {
        return Err(PartyChangeOwnerError::InvalidNewOwnerEntity);
    }

    let mut party = party_query
        .get_mut(party_entity)
        .expect("PartyMembership pointing to invalid party entity");

    if party.owner != owner_entity {
        return Err(PartyChangeOwnerError::NotOwner);
    }
    party.owner = new_owner_entity;

    // Inform party members of new owner
    send_message_to_members(
        party_member_info_query,
        &party.members,
        ServerMessage::PartyChangeOwner(new_owner.client_entity.id),
        None,
    );

    Ok(())
}

enum PartyUpdateRulesError {
    InvalidEntity,
    NotOwner,
    NotInParty,
}

fn handle_party_update_rules(
    party_query: &mut Query<&mut Party>,
    party_membership_query: &Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    owner_entity: Entity,
    item_sharing: PartyItemSharing,
    xp_sharing: PartyXpSharing,
) -> Result<(), PartyUpdateRulesError> {
    let party_membership = party_membership_query
        .get(owner_entity)
        .map_err(|_| PartyUpdateRulesError::InvalidEntity)?;

    let party_entity = match *party_membership.party_membership {
        PartyMembership::None => return Err(PartyUpdateRulesError::NotInParty),
        PartyMembership::Member(party_entity) => party_entity,
    };

    let mut party = party_query
        .get_mut(party_entity)
        .expect("PartyMembership pointing to invalid party entity");

    if party.owner != owner_entity {
        return Err(PartyUpdateRulesError::NotOwner);
    }

    party.item_sharing = item_sharing;
    party.xp_sharing = xp_sharing;

    send_message_to_members(
        party_member_info_query,
        &party.members,
        ServerMessage::PartyUpdateRules(item_sharing, xp_sharing),
        None,
    );

    Ok(())
}

enum PartyMemberDisconnectError {
    InvalidParty,
}

fn handle_party_member_disconnect(
    commands: &mut Commands,
    party_query: &mut Query<&mut Party>,
    party_membership_query: &mut Query<PartyMembershipQuery>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    party_entity: Entity,
    disconnect_entity: Entity,
    character_id: CharacterUniqueId,
    name: String,
) -> Result<(), PartyMemberDisconnectError> {
    let mut party = party_query
        .get_mut(party_entity)
        .map_err(|_| PartyMemberDisconnectError::InvalidParty)?;

    // Set the member to offline
    for party_member in party.members.iter_mut() {
        if let PartyMember::Online(member_entity) = party_member {
            if *member_entity == disconnect_entity {
                *party_member = PartyMember::Offline(character_id, name);
                break;
            }
        }
    }

    // If leader disconnects, change leader to first online member, or disband party if all offline
    if party.owner == disconnect_entity {
        let new_owner_entity = party.members.iter().find_map(|party_member| {
            if let PartyMember::Online(entity) = party_member {
                Some(*entity)
            } else {
                None
            }
        });

        if let Some(new_owner) = new_owner_entity
            .and_then(|new_owner_entity| party_membership_query.get(new_owner_entity).ok())
        {
            party.owner = new_owner_entity.unwrap();
            send_message_to_members(
                party_member_info_query,
                &party.members,
                ServerMessage::PartyChangeOwner(new_owner.client_entity.id),
                None,
            );
        } else {
            // No other online players, delete party
            delete_party(commands, party_membership_query, party_entity, &mut party);
            return Ok(());
        }
    }

    // Send disconnect message to all online members
    send_message_to_members(
        party_member_info_query,
        &party.members,
        ServerMessage::PartyMemberDisconnect(character_id),
        None,
    );

    Ok(())
}

enum PartyMemberReconnectError {
    InvalidParty,
}

fn handle_party_member_reconnect(
    party_query: &Query<&mut Party>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    party_entity: Entity,
    reconnect_entity: Entity,
    character_id: CharacterUniqueId,
    name: String,
) -> Result<(), PartyMemberDisconnectError> {
    let party = party_query
        .get(party_entity)
        .map_err(|_| PartyMemberDisconnectError::InvalidParty)?;

    // Send member list to reconnected member
    if let Ok(reconnect_member) = party_member_info_query.get(reconnect_entity) {
        let owner = party_member_info_query.get(party.owner).unwrap();
        let owner_character_id = owner.character_info.unique_id;

        let party_member_infos = get_party_membership_info(&party.members, party_member_info_query);
        let other_members_info = party_member_infos
            .into_iter()
            .filter(|member_info| {
                if let PartyMemberInfo::Online(party_member_online) = member_info {
                    party_member_online.character_id != character_id
                        && party_member_online.name != name
                } else {
                    true
                }
            })
            .collect();

        if let Some(game_client) = reconnect_member.game_client {
            game_client
                .server_message_tx
                .send(ServerMessage::PartyMemberList(PartyMemberList {
                    item_sharing: party.item_sharing,
                    xp_sharing: party.xp_sharing,
                    owner_character_id,
                    members: other_members_info,
                }))
                .ok();
        }
    }

    Ok(())
}

enum PartyMemberUpdateInfoError {
    InvalidParty,
}

fn handle_party_member_update_info(
    party_query: &mut Query<&mut Party>,
    party_member_info_query: &Query<PartyMemberInfoQuery>,
    party_entity: Entity,
    member_entity: Entity,
) -> Result<(), PartyMemberUpdateInfoError> {
    let party = party_query
        .get(party_entity)
        .map_err(|_| PartyMemberUpdateInfoError::InvalidParty)?;

    if let Some(member_info) = get_online_party_member_info(party_member_info_query, member_entity)
    {
        send_message_to_members(
            party_member_info_query,
            &party.members,
            ServerMessage::PartyMemberUpdateInfo(member_info),
            Some(member_entity),
        );
    }

    Ok(())
}

pub fn party_member_event_system(
    mut commands: Commands,
    mut party_query: Query<&mut Party>,
    mut party_membership_query: Query<PartyMembershipQuery>,
    party_member_info_query: Query<PartyMemberInfoQuery>,
    mut party_member_events: EventReader<PartyMemberEvent>,
) {
    for event in party_member_events.iter() {
        match event {
            PartyMemberEvent::Disconnect(PartyMemberDisconnect {
                party_entity,
                disconnect_entity,
                character_id,
                name,
            }) => {
                handle_party_member_disconnect(
                    &mut commands,
                    &mut party_query,
                    &mut party_membership_query,
                    &party_member_info_query,
                    *party_entity,
                    *disconnect_entity,
                    *character_id,
                    name.clone(),
                )
                .ok();
            }
            PartyMemberEvent::Reconnect(PartyMemberReconnect {
                party_entity,
                reconnect_entity,
                character_id,
                name,
            }) => {
                handle_party_member_reconnect(
                    &party_query,
                    &party_member_info_query,
                    *party_entity,
                    *reconnect_entity,
                    *character_id,
                    name.clone(),
                )
                .ok();
            }
        }
    }
}

pub fn party_system(
    mut commands: Commands,
    mut party_query: Query<&mut Party>,
    mut party_membership_query: Query<PartyMembershipQuery>,
    party_member_info_query: Query<PartyMemberInfoQuery>,
    mut party_events: EventReader<PartyEvent>,
) {
    for event in party_events.iter() {
        match *event {
            PartyEvent::Invite(PartyEventInvite {
                owner_entity,
                invited_entity,
            }) => {
                handle_party_invite(&mut party_membership_query, owner_entity, invited_entity).ok();
            }
            PartyEvent::AcceptInvite(PartyEventInvite {
                owner_entity,
                invited_entity,
            }) => {
                handle_party_accept_invite(
                    &mut commands,
                    &mut party_query,
                    &mut party_membership_query,
                    &party_member_info_query,
                    owner_entity,
                    invited_entity,
                )
                .ok();
            }
            PartyEvent::RejectInvite(
                reason,
                PartyEventInvite {
                    owner_entity,
                    invited_entity,
                },
            ) => {
                handle_party_reject_invite(
                    &mut party_membership_query,
                    reason,
                    owner_entity,
                    invited_entity,
                )
                .ok();
            }
            PartyEvent::Leave(PartyEventLeave { leaver_entity }) => {
                handle_party_leave(
                    &mut commands,
                    &mut party_query,
                    &mut party_membership_query,
                    &party_member_info_query,
                    leaver_entity,
                )
                .ok();
            }
            PartyEvent::Kick(PartyEventKick {
                owner_entity,
                kick_character_id,
            }) => {
                handle_party_kick(
                    &mut commands,
                    &mut party_query,
                    &mut party_membership_query,
                    &party_member_info_query,
                    owner_entity,
                    kick_character_id,
                )
                .ok();
            }
            PartyEvent::ChangeOwner(PartyEventChangeOwner {
                owner_entity,
                new_owner_entity,
            }) => {
                handle_party_change_owner(
                    &mut party_query,
                    &mut party_membership_query,
                    &party_member_info_query,
                    owner_entity,
                    new_owner_entity,
                )
                .ok();
            }
            PartyEvent::UpdateRules(PartyEventUpdateRules {
                owner_entity,
                item_sharing,
                xp_sharing,
            }) => {
                handle_party_update_rules(
                    &mut party_query,
                    &party_membership_query,
                    &party_member_info_query,
                    owner_entity,
                    item_sharing,
                    xp_sharing,
                )
                .ok();
            }
        }
    }
}

pub fn party_member_update_info_system(
    party_query: Query<&Party>,
    party_member_info_query: Query<PartyMemberInfoQuery>,
    party_member_info_changed_query: Query<
        (Entity, &PartyMembership),
        Or<(
            Changed<AbilityValues>,
            Changed<ClientEntity>,
            Changed<StatusEffects>,
        )>,
    >,
) {
    for (member_entity, party_membership) in party_member_info_changed_query.iter() {
        if let &PartyMembership::Member(party_entity) = party_membership {
            if let Ok(party) = party_query.get(party_entity) {
                if let Some(member_info) =
                    get_online_party_member_info(&party_member_info_query, member_entity)
                {
                    send_message_to_members(
                        &party_member_info_query,
                        &party.members,
                        ServerMessage::PartyMemberUpdateInfo(member_info),
                        Some(member_entity),
                    );
                }
            }
        }
    }
}

pub fn party_update_average_level_system(
    mut query_party: Query<&mut Party>,
    query_level: Query<&Level>,
) {
    for mut party in query_party.iter_mut() {
        let mut total_levels = 0;
        let mut num_online = 0;

        for member in party.members.iter() {
            if let &PartyMember::Online(entity) = member {
                if let Ok(level) = query_level.get(entity) {
                    total_levels += level.level;
                    num_online += 1;
                }
            }
        }

        if num_online != 0 {
            party.average_member_level = total_levels as i32 / num_online;
        }
    }
}
