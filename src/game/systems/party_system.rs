use bevy_ecs::{
    prelude::{Changed, Commands, Entity, EventReader, Or, Query, QueryState},
    system::QuerySet,
};

use crate::game::{
    components::{
        AbilityValues, CharacterInfo, CharacterUniqueId, ClientEntity, GameClient, HealthPoints,
        Party, PartyMember, PartyMembership, Stamina, StatusEffects,
    },
    events::{
        PartyEvent, PartyEventChangeOwner, PartyEventInvite, PartyEventKick, PartyEventLeave,
        PartyMemberDisconnect, PartyMemberReconnect,
    },
    messages::server::{
        PartyMemberInfo, PartyMemberInfoOffline, PartyMemberInfoOnline, PartyMemberLeave,
        PartyMemberList, PartyReply, PartyRequest, ServerMessage,
    },
};

/*
Party event system TODO:
- Check party level / team requirements?
*/

fn send_message_to_members(
    game_client_query: &Query<&GameClient>,
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

            if let Ok(game_client) = game_client_query.get(*party_member_entity) {
                game_client.server_message_tx.send(message.clone()).ok();
            }
        }
    }
}

fn delete_party(
    commands: &mut Commands,
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    party_entity: Entity,
    party: &mut Party,
) {
    for party_member in party.members.iter() {
        if let PartyMember::Online(party_member_entity) = party_member {
            let (_, _, mut member_party_membership, member_game_client) =
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
    commands.entity(party_entity).despawn();
}

enum PartyInviteError {
    AlreadyHasParty,
    NoPermission,
    PartyFull,
    InvalidEntity,
}

fn handle_party_invite(
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let (owner_client_entity, _, owner_party_membership, _) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let is_create_party = matches!(*owner_party_membership, PartyMembership::None);
    let owner_client_entity_id = owner_client_entity.id;

    let (_, _, invited_party_membership, invited_game_client) = party_member_query
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

fn get_online_party_member_info(
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    entity: Entity,
) -> Option<PartyMemberInfoOnline> {
    let (ability_values, character_info, client_entity, health_points, status_effects, stamina) =
        party_member_info_query.get(entity).ok()?;

    Some(PartyMemberInfoOnline {
        character_id: character_info.unique_id,
        name: character_info.name.clone(),
        entity_id: client_entity.id,
        health_points: *health_points,
        status_effects: status_effects.active.clone(),
        max_health: ability_values.get_max_health(),
        concentration: ability_values.get_concentration(),
        health_recovery: ability_values.get_additional_health_recovery(), // TODO: ??
        mana_recovery: ability_values.get_additional_mana_recovery(),     // TODO: ??
        stamina: *stamina,
    })
}

fn get_party_membership_info(
    party_members: &[PartyMember],
    party_member_info_query: &Query<(
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
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: &Query<&GameClient>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    // First ensure invited entity is not already in a party
    let (invited_client_entity, _, invited_party_membership, _) = party_member_query
        .get_mut(invited_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    if !matches!(*invited_party_membership, PartyMembership::None) {
        return Err(PartyInviteError::AlreadyHasParty);
    }
    let invited_client_entity_id = invited_client_entity.id;

    let (_, owner_character_info, mut owner_party_membership, _) = party_member_query
        .get_mut(owner_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let is_create_party = matches!(*owner_party_membership, PartyMembership::None);
    let owner_character_id = owner_character_info.unique_id;

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

            let (_, _, mut invited_party_membership, _) =
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

            let (_, _, mut invited_party_membership, _) =
                party_member_query.get_mut(invited_entity).unwrap();
            *invited_party_membership = PartyMembership::new(party_entity);

            party.members.clone()
        }
    };

    // Send accept create to owner
    if is_create_party {
        let (_, _, _, owner_game_client) = party_member_query.get_mut(owner_entity).unwrap();
        if let Some(owner_game_client) = owner_game_client {
            owner_game_client
                .server_message_tx
                .send(ServerMessage::PartyReply(PartyReply::AcceptCreate(
                    invited_client_entity_id,
                )))
                .ok();
        }
    }

    let party_member_infos = get_party_membership_info(&party_members, party_member_info_query);
    let (invited_member_info, other_members_info): (Vec<_>, Vec<_>) =
        party_member_infos.into_iter().partition(|member_info| {
            if let PartyMemberInfo::Online(online_party_member) = member_info {
                online_party_member.entity_id == invited_client_entity_id
            } else {
                false
            }
        });

    // Send list of other members to invited
    let (_, _, _, invited_game_client) = party_member_query.get_mut(invited_entity).unwrap();
    if let Some(invited_game_client) = invited_game_client {
        invited_game_client
            .server_message_tx
            .send(ServerMessage::PartyMemberList(PartyMemberList {
                owner_character_id,
                members: other_members_info,
            }))
            .ok();
    }

    // Send info about invited to other members
    send_message_to_members(
        game_client_query,
        &party_members,
        ServerMessage::PartyMemberList(PartyMemberList {
            owner_character_id,
            members: invited_member_info,
        }),
        Some(invited_entity),
    );

    Ok(())
}

fn handle_party_reject_invite(
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    owner_entity: Entity,
    invited_entity: Entity,
) -> Result<(), PartyInviteError> {
    let (invited_client_entity, _, _, _) = party_member_query
        .get_mut(invited_entity)
        .map_err(|_| PartyInviteError::InvalidEntity)?;
    let invited_client_entity_id = invited_client_entity.id;

    let (_, _, _, owner_game_client) = party_member_query
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
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: &Query<&GameClient>,
    leaver_entity: Entity,
) -> Result<(), PartyLeaveError> {
    let (_, _, mut leaver_party_membership, leaver_game_client) = party_member_query
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
                delete_party(commands, party_member_query, party_entity, &mut party);
            } else {
                // Get leaver character id and owner character id for leave message
                let (_, leaver_character_info, _, _, _, _) =
                    party_member_info_query.get(leaver_entity).unwrap();
                let leaver_character_id = leaver_character_info.unique_id;

                let (_, owner_character_info, _, _, _, _) =
                    party_member_info_query.get(party.owner).unwrap();
                let owner_character_id = owner_character_info.unique_id;

                // Send message to other members informing of leaver and new owner
                send_message_to_members(
                    game_client_query,
                    &party.members,
                    ServerMessage::PartyMemberLeave(PartyMemberLeave {
                        leaver_character_id,
                        owner_character_id,
                    }),
                    None,
                );
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
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: &Query<&GameClient>,
    owner_entity: Entity,
    kick_character_id: CharacterUniqueId,
) -> Result<(), PartyKickError> {
    // First ensure owner_entity is actually owner of the party
    let (_, _, owner_party_membership, _) = party_member_query
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
        .get(owner_entity)
        .map_err(|_| PartyKickError::InvalidEntity)?;
    if owner_character_info.unique_id == kick_character_id {
        return Err(PartyKickError::InvalidKickCharacter);
    }

    // Try to remove kicked member from party
    let mut kicked_online_entity = None;
    let mut kicked_offline = false;
    party.members.retain(|party_member| match *party_member {
        PartyMember::Online(party_member_entity) => party_member_info_query
            .get(party_member_entity)
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
        let (_, _, mut kicked_party_membership, kicked_game_client) =
            party_member_query.get_mut(kicked_entity).unwrap();

        *kicked_party_membership = PartyMembership::None;

        if let Some(kicked_game_client) = kicked_game_client {
            kicked_game_client
                .server_message_tx
                .send(ServerMessage::PartyMemberKicked(kick_character_id))
                .ok();
        }
    }

    // Send kick message to other party member
    send_message_to_members(
        game_client_query,
        &party.members,
        ServerMessage::PartyMemberKicked(kick_character_id),
        None,
    );

    // If party is down to 1 member, delete the party
    if party.members.len() <= 1 {
        delete_party(commands, party_member_query, party_entity, &mut party);
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
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    game_client_query: &Query<&GameClient>,
    owner_entity: Entity,
    new_owner_entity: Entity,
) -> Result<(), PartyChangeOwnerError> {
    // Get owner entity's party
    let (_, _, owner_party_membership, _) = party_member_query
        .get(owner_entity)
        .map_err(|_| PartyChangeOwnerError::InvalidEntity)?;
    let party_entity = match *owner_party_membership {
        PartyMembership::None => return Err(PartyChangeOwnerError::NotInParty),
        PartyMembership::Member(party_entity) => party_entity,
    };

    // Ensure new owner is in the same party
    let (new_owner_client_entity, _, new_owner_party_membership, _) = party_member_query
        .get(new_owner_entity)
        .map_err(|_| PartyChangeOwnerError::InvalidEntity)?;
    let new_owner_party_entity = match *new_owner_party_membership {
        PartyMembership::None => return Err(PartyChangeOwnerError::InvalidNewOwnerEntity),
        PartyMembership::Member(new_owner_party_entity) => new_owner_party_entity,
    };
    let new_owner_client_entity_id = new_owner_client_entity.id;
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
        game_client_query,
        &party.members,
        ServerMessage::PartyChangeOwner(new_owner_client_entity_id),
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
    party_member_query: &mut Query<(
        &ClientEntity,
        &CharacterInfo,
        &mut PartyMembership,
        Option<&GameClient>,
    )>,
    game_client_query: &Query<&GameClient>,
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
        let new_owner = party.members.iter().find_map(|party_member| {
            if let PartyMember::Online(entity) = party_member {
                Some(*entity)
            } else {
                None
            }
        });

        if let Some((new_owner_client_entity, _, _, _)) =
            new_owner.and_then(|new_owner| party_member_query.get(new_owner).ok())
        {
            party.owner = new_owner.unwrap();
            send_message_to_members(
                game_client_query,
                &party.members,
                ServerMessage::PartyChangeOwner(new_owner_client_entity.id),
                None,
            );
        } else {
            // No other online players, delete party
            delete_party(commands, party_member_query, party_entity, &mut party);
            return Ok(());
        }
    }

    // Send disconnect message to all online members
    send_message_to_members(
        game_client_query,
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
    party_query: &mut Query<&mut Party>,
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: &Query<&GameClient>,
    party_entity: Entity,
    reconnect_entity: Entity,
    character_id: CharacterUniqueId,
    name: String,
) -> Result<(), PartyMemberDisconnectError> {
    let mut party = party_query
        .get_mut(party_entity)
        .map_err(|_| PartyMemberDisconnectError::InvalidParty)?;

    // Send member list to reconnected member
    if let Ok(game_client) = game_client_query.get(reconnect_entity) {
        let (_, owner_character_info, _, _, _, _) =
            party_member_info_query.get(party.owner).unwrap();
        let owner_character_id = owner_character_info.unique_id;

        let party_member_infos = get_party_membership_info(&party.members, party_member_info_query);
        let other_members_info = party_member_infos
            .into_iter()
            .filter(|member_info| {
                if let PartyMemberInfo::Offline(party_member_offline) = member_info {
                    party_member_offline.character_id != character_id
                        && party_member_offline.name != name
                } else {
                    true
                }
            })
            .collect();

        game_client
            .server_message_tx
            .send(ServerMessage::PartyMemberList(PartyMemberList {
                owner_character_id,
                members: other_members_info,
            }))
            .ok();
    }

    // Set the member to online
    for party_member in party.members.iter_mut() {
        if let PartyMember::Offline(party_member_character_id, party_member_name) = party_member {
            if *party_member_character_id == character_id && party_member_name == &name {
                *party_member = PartyMember::Online(reconnect_entity);
                break;
            }
        }
    }

    Ok(())
}
enum PartyMemberUpdateInfoError {
    InvalidParty,
}

fn handle_party_member_update_info(
    party_query: &mut Query<&mut Party>,
    party_member_info_query: &Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: &Query<&GameClient>,
    party_entity: Entity,
    member_entity: Entity,
) -> Result<(), PartyMemberUpdateInfoError> {
    let party = party_query
        .get(party_entity)
        .map_err(|_| PartyMemberUpdateInfoError::InvalidParty)?;

    if let Some(member_info) = get_online_party_member_info(party_member_info_query, member_entity)
    {
        send_message_to_members(
            game_client_query,
            &party.members,
            ServerMessage::PartyMemberUpdateInfo(member_info),
            Some(member_entity),
        );
    }

    Ok(())
}

pub fn party_system(
    mut commands: Commands,
    mut party_query: Query<&mut Party>,
    mut party_member_query_set: QuerySet<(
        QueryState<(
            &ClientEntity,
            &CharacterInfo,
            &mut PartyMembership,
            Option<&GameClient>,
        )>,
        QueryState<
            (Entity, &PartyMembership),
            Or<(
                Changed<AbilityValues>,
                Changed<ClientEntity>,
                Changed<StatusEffects>,
            )>,
        >,
    )>,
    party_member_info_query: Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &HealthPoints,
        &StatusEffects,
        &Stamina,
    )>,
    game_client_query: Query<&GameClient>,
    mut party_events: EventReader<PartyEvent>,
) {
    // Collect party events into a Vec so we can iterate disconnects separately
    let party_events: Vec<PartyEvent> = party_events.iter().cloned().collect();

    // First handle member disconnects / reconnects
    let mut party_member_query = party_member_query_set.q0();
    for event in party_events.iter() {
        match event {
            PartyEvent::MemberDisconnect(PartyMemberDisconnect {
                party_entity,
                disconnect_entity,
                character_id,
                name,
            }) => {
                handle_party_member_disconnect(
                    &mut commands,
                    &mut party_query,
                    &mut party_member_query,
                    &game_client_query,
                    *party_entity,
                    *disconnect_entity,
                    *character_id,
                    name.clone(),
                )
                .ok();
            }
            PartyEvent::MemberReconnect(PartyMemberReconnect {
                party_entity,
                reconnect_entity,
                character_id,
                name,
            }) => {
                handle_party_member_reconnect(
                    &mut party_query,
                    &party_member_info_query,
                    &game_client_query,
                    *party_entity,
                    *reconnect_entity,
                    *character_id,
                    name.clone(),
                )
                .ok();
            }
            _ => {}
        }
    }

    // Then handle remaining party events
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
                    &party_member_info_query,
                    &game_client_query,
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
                    &party_member_info_query,
                    &game_client_query,
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
                    &party_member_info_query,
                    &game_client_query,
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
                    &mut party_member_query,
                    &game_client_query,
                    owner_entity,
                    new_owner_entity,
                )
                .ok();
            }
            PartyEvent::MemberDisconnect(_) => {}
            PartyEvent::MemberReconnect(_) => {}
        }
    }

    let party_member_info_changed_query = party_member_query_set.q1();
    for (member_entity, party_membership) in party_member_info_changed_query.iter() {
        if let PartyMembership::Member(party_entity) = party_membership {
            handle_party_member_update_info(
                &mut party_query,
                &party_member_info_query,
                &game_client_query,
                *party_entity,
                member_entity,
            )
            .ok();
        }
    }
}
