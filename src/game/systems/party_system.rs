use bevy_ecs::prelude::{Commands, Entity, EventReader, Query};
use log::warn;

use crate::game::{
    components::{
        AbilityValues, CharacterInfo, CharacterUniqueId, ClientEntity, GameClient, HealthPoints,
        Party, PartyMember, PartyMembership, Stamina, StatusEffects,
    },
    events::{PartyEvent, PartyEventInvite, PartyEventKick, PartyEventLeave, PartyEventChangeOwner},
    messages::server::{
        PartyMemberInfo, PartyMemberInfoOffline, PartyMemberInfoOnline, PartyMemberLeave,
        PartyReply, PartyRequest, ServerMessage,
    },
};

/*
Party event system TODO:
- Handle member disconnect
- If leader disconnects, change leader
- If all players disconnect the party must disband
- Change party owner
- Check party level / team requirements?
*/

enum PartyInviteError {
    AlreadyHasParty,
    NoPermission,
    PartyFull,
    InvalidEntity,
}

fn handle_party_invite(
    party_member_query: &mut Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let (owner_client_entity, owner_party_membership, _) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let is_create_party = matches!(*owner_party_membership, PartyMembership::None);
    let owner_client_entity_id = owner_client_entity.id;

    let (_, invited_party_membership, invited_game_client) = party_member_query
        .get_mut(invited_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    if !matches!(*invited_party_membership, PartyMembership::None) {
        return Err(PartyInviteError::AlreadyHasParty);
    }

    if let Some(invited_game_client) = invited_game_client {
        let party_request = if is_create_party {
            PartyRequest::Create(owner_client_entity_id)
        } else {
            PartyRequest::Invite(owner_client_entity_id)
        };

        invited_game_client
            .server_message_tx
            .send(ServerMessage::PartyRequest(party_request))
            .ok();
    }

    Ok(())
}

fn get_party_membership_info(
    party_members: &[PartyMember],
    party_member_info_query: &mut Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
) -> Vec<PartyMemberInfo> {
    let mut info = Vec::new();
    for party_member in party_members.iter() {
        match party_member {
            &PartyMember::Online(party_member_entity) => {
                if let Ok((
                    ability_values,
                    character_info,
                    client_entity,
                    health_points,
                    status_effects,
                    stamina,
                )) = party_member_info_query.get_mut(party_member_entity)
                {
                    info.push(PartyMemberInfo::Online(PartyMemberInfoOnline {
                        character_id: character_info.unique_id,
                        name: character_info.name.clone(),
                        entity_id: client_entity.id,
                        health_points: *health_points,
                        status_effects: status_effects.clone(),
                        max_health: ability_values.get_max_health(),
                        concentration: ability_values.get_concentration(),
                        health_recovery: ability_values.get_additional_health_recovery(), // TODO: ??
                        mana_recovery: ability_values.get_additional_mana_recovery(), // TODO: ??
                        stamina: *stamina,
                    }));
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
    party_member_query: &mut Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    mut party_member_info_query: &mut Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    // First ensure invited entity is not already in a party
    let (invited_client_entity, invited_party_membership, _) = party_member_query
        .get_mut(invited_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    if !matches!(*invited_party_membership, PartyMembership::None) {
        return Err(PartyInviteError::AlreadyHasParty);
    }
    let invited_client_entity_id = invited_client_entity.id;

    let (_, mut owner_party_membership, _) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let is_create_party = matches!(*owner_party_membership, PartyMembership::None);

    let party_members = match *owner_party_membership {
        PartyMembership::None => {
            // Create a new party
            let party = Party::new(
                owner_entity,
                &[
                    PartyMember::Online(owner_entity),
                    PartyMember::Online(invited_entity),
                ],
            );
            let party_members = party.members.clone();
            let party_entity = commands.spawn().insert(party).id();
            *owner_party_membership = PartyMembership::new(party_entity);

            let (_, mut invited_party_membership, _) =
                party_member_query.get_mut(invited_entity).unwrap();
            *invited_party_membership = PartyMembership::new(party_entity);

            party_members
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

            let (_, mut invited_party_membership, _) =
                party_member_query.get_mut(invited_entity).unwrap();
            *invited_party_membership = PartyMembership::new(party_entity);

            party.members.clone()
        }
    };

    // Send accept create to owner
    if is_create_party {
        let (_, _, owner_game_client) = party_member_query.get_mut(owner_entity).unwrap();
        if let Some(owner_game_client) = owner_game_client {
            owner_game_client
                .server_message_tx
                .send(ServerMessage::PartyReply(PartyReply::AcceptCreate(
                    invited_client_entity_id,
                )))
                .ok();
        }
    }

    let party_member_infos =
        get_party_membership_info(&party_members, &mut party_member_info_query);
    let (invited_member_info, other_members_info): (Vec<_>, Vec<_>) =
        party_member_infos.into_iter().partition(|member_info| {
            if let PartyMemberInfo::Online(online_party_member) = member_info {
                online_party_member.entity_id == invited_client_entity_id
            } else {
                false
            }
        });

    // Send list of other members to invited
    let (_, _, invited_game_client) = party_member_query.get_mut(invited_entity).unwrap();
    if let Some(invited_game_client) = invited_game_client {
        invited_game_client
            .server_message_tx
            .send(ServerMessage::PartyMemberList(other_members_info))
            .ok();
    }

    // Send info about invited to other members
    for party_member in party_members.iter() {
        if let PartyMember::Online(party_member_entity) = party_member {
            let (member_client_entity, _, member_game_client) =
                party_member_query.get_mut(*party_member_entity).unwrap();

            if member_client_entity.id == invited_client_entity_id {
                continue;
            }

            if let Some(member_game_client) = member_game_client {
                member_game_client
                    .server_message_tx
                    .send(ServerMessage::PartyMemberList(invited_member_info.clone()))
                    .ok();
            }
        }
    }

    Ok(())
}

fn handle_party_reject_invite(
    party_member_query: &mut Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let (invited_client_entity, _, _) = party_member_query
        .get_mut(invited_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let invited_client_entity_id = invited_client_entity.id;

    let (_, _, owner_game_client) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;

    if let Some(owner_game_client) = owner_game_client {
        owner_game_client
            .server_message_tx
            .send(ServerMessage::PartyReply(PartyReply::RejectInvite(
                invited_client_entity_id,
            )))
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
    party_member_query: &mut Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    party_member_info_query: &mut Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    leaver_entity: Entity,
) -> Result<(), PartyLeaveError> {
    let (_, mut leaver_party_membership, leaver_game_client) = party_member_query
        .get_mut(leaver_entity)
        .map_err(|_| PartyLeaveError::InvalidEntity)?;

    if let Some(leaver_game_client) = leaver_game_client {
        // Send party delete message to leaver
        leaver_game_client
            .server_message_tx
            .send(ServerMessage::PartyReply(PartyReply::DeleteParty))
            .ok();
    }

    match *leaver_party_membership {
        PartyMembership::Member(party_entity) => {
            // Remove leaver from party
            let mut party = party_query
                .get_mut(party_entity)
                .expect("PartyMembership pointing to invalid party entity");
            party.members.retain(|party_member| match party_member {
                PartyMember::Online(party_member_entity) => *party_member_entity != leaver_entity,
                _ => true,
            });
            *leaver_party_membership = PartyMembership::None;

            if party.owner == leaver_entity {
                // If owner is leaving, choose first found online member to be new owner
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
                // Send delete message to other members and remove them from party
                for party_member in party.members.iter() {
                    if let PartyMember::Online(party_member_entity) = party_member {
                        let (_, mut member_party_membership, member_game_client) =
                            party_member_query.get_mut(*party_member_entity).unwrap();
                        if let Some(member_game_client) = member_game_client {
                            member_game_client
                                .server_message_tx
                                .send(ServerMessage::PartyReply(PartyReply::DeleteParty))
                                .ok();
                        }

                        *member_party_membership = PartyMembership::None;
                    }
                }
                party.members.clear();

                // Delete the party
                commands.entity(party_entity).despawn();
            } else {
                // Get leaver character id and owner character id for leave message
                let (_, leaver_character_info, _, _, _, _) =
                    party_member_info_query.get(leaver_entity).unwrap();
                let leaver_character_id = leaver_character_info.unique_id;

                let (_, owner_character_info, _, _, _, _) =
                    party_member_info_query.get(party.owner).unwrap();
                let owner_character_id = owner_character_info.unique_id;

                // Send message to other members informing of leaver and new owner
                for party_member in party.members.iter() {
                    if let PartyMember::Online(party_member_entity) = party_member {
                        let (_, _, member_game_client) =
                            party_member_query.get_mut(*party_member_entity).unwrap();
                        if let Some(member_game_client) = member_game_client {
                            member_game_client
                                .server_message_tx
                                .send(ServerMessage::PartyMemberLeave(PartyMemberLeave {
                                    leaver_character_id,
                                    owner_character_id,
                                }))
                                .ok();
                        }
                    }
                }
            }
        }
        PartyMembership::None => return Err(PartyLeaveError::NotInParty),
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
    party_member_query: &mut Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    party_member_info_query: &mut Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    owner_entity: Entity,
    kick_character_id: CharacterUniqueId,
) -> Result<(), PartyKickError> {
    // First ensure owner_entity is actually owner of the party
    let (_, owner_party_membership, _) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyKickError::InvalidEntity)?;
    let party_entity = match *owner_party_membership {
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
    let (_, owner_character_info, _, _, _, _) = party_member_info_query
        .get_mut(owner_entity)
        .map_err(|_| PartyKickError::InvalidEntity)?;
    if owner_character_info.unique_id == kick_character_id {
        return Err(PartyKickError::InvalidKickCharacter);
    }

    // Try to remove kicked member from party
    let mut kicked_online_entity = None;
    let mut kicked_offline = false;
    party.members.retain(|party_member| match *party_member {
        PartyMember::Online(party_member_entity) => party_member_info_query
            .get_mut(party_member_entity)
            .map_or(true, |(_, party_member_character_info, _, _, _, _)| {
                if party_member_character_info.unique_id == kick_character_id {
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
        let (_, mut kicked_party_membership, kicked_game_client) =
            party_member_query.get_mut(kicked_entity).unwrap();

        *kicked_party_membership = PartyMembership::None;

        if let Some(kicked_game_client) = kicked_game_client {
            kicked_game_client
                .server_message_tx
                .send(ServerMessage::PartyMemberKicked(kick_character_id))
                .ok();
        }
    }

    // Send kick message to other party members
    for party_member in party.members.iter() {
        if let PartyMember::Online(party_member_entity) = party_member {
            let (_, _, member_game_client) =
                party_member_query.get_mut(*party_member_entity).unwrap();
            if let Some(member_game_client) = member_game_client {
                member_game_client
                    .server_message_tx
                    .send(ServerMessage::PartyMemberKicked(kick_character_id))
                    .ok();
            }
        }
    }

    // If party is down to 1 member, delete the party
    if party.members.len() <= 1 {
        for party_member in party.members.iter() {
            if let PartyMember::Online(party_member_entity) = party_member {
                let (_, mut member_party_membership, member_game_client) =
                    party_member_query.get_mut(*party_member_entity).unwrap();
                if let Some(member_game_client) = member_game_client {
                    member_game_client
                        .server_message_tx
                        .send(ServerMessage::PartyReply(PartyReply::DeleteParty))
                        .ok();
                }

                *member_party_membership = PartyMembership::None;
            }
        }
        party.members.clear();

        // Delete the party
        commands.entity(party_entity).despawn();
    }

    Ok(())
}

pub fn party_system(
    mut commands: Commands,
    mut party_query: Query<&mut Party>,
    mut party_member_query: Query<(&ClientEntity, &mut PartyMembership, Option<&GameClient>)>,
    mut party_member_info_query: Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    mut party_events: EventReader<PartyEvent>,
) {
    for event in party_events.iter() {
        match *event {
            PartyEvent::Invite(PartyEventInvite {
                owner_entity,
                invited_entity,
            }) => {
                handle_party_invite(&mut party_member_query, owner_entity, invited_entity).ok();
            }
            PartyEvent::AcceptInvite(PartyEventInvite {
                owner_entity,
                invited_entity,
            }) => {
                handle_party_accept_invite(
                    &mut commands,
                    &mut party_query,
                    &mut party_member_query,
                    &mut party_member_info_query,
                    owner_entity,
                    invited_entity,
                )
                .ok();
            }
            PartyEvent::RejectInvite(PartyEventInvite {
                owner_entity,
                invited_entity,
            }) => {
                handle_party_reject_invite(&mut party_member_query, owner_entity, invited_entity)
                    .ok();
            }
            PartyEvent::Leave(PartyEventLeave { leaver_entity }) => {
                handle_party_leave(
                    &mut commands,
                    &mut party_query,
                    &mut party_member_query,
                    &mut party_member_info_query,
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
                    &mut party_member_query,
                    &mut party_member_info_query,
                    owner_entity,
                    kick_character_id,
                )
                .ok();
            }
            PartyEvent::ChangeOwner(PartyEventChangeOwner {
                owner_entity: _owner_entity,
                new_owner_entity: _new_owner_entity,
            }) => {
                warn!("Unimplemented PartyEvent::ChangeOwner.");
            }
        }
    }
}
