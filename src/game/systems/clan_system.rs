use std::num::{NonZeroU32, NonZeroUsize};

use bevy::{
    ecs::query::WorldQuery,
    prelude::{Changed, Commands, Entity, EventReader, Query, ResMut},
};

use rose_data::{ClanMemberPosition, QuestTriggerHash};
use rose_game_common::{
    components::{ClanLevel, ClanPoints, ClanUniqueId},
    messages::server::{ClanCreateError, ClanMemberInfo, ServerMessage},
};

use crate::game::{
    components::{
        CharacterInfo, Clan, ClanMember, ClanMembership, ClientEntity, GameClient, Inventory,
        Level, Money,
    },
    events::ClanEvent,
    resources::ServerMessages,
    storage::clan::{ClanStorage, ClanStorageMember},
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct CreatorQuery<'w> {
    client_entity: &'w ClientEntity,
    character_info: &'w CharacterInfo,
    level: &'w Level,
    inventory: &'w mut Inventory,
    game_client: Option<&'w GameClient>,
    clan_membership: Option<&'w ClanMembership>,
}

#[derive(WorldQuery)]
pub struct MemberQuery<'w> {
    entity: Entity,
    character_info: &'w CharacterInfo,
    clan_membership: &'w ClanMembership,
    level: &'w Level,
    game_client: Option<&'w GameClient>,
}

fn send_update_clan_info(clan: &Clan, query_member: &Query<MemberQuery>) {
    for clan_member in clan.members.iter() {
        let &ClanMember::Online { entity: clan_member_entity, .. } = clan_member else {
            continue;
        };

        if let Ok(online_member) = query_member.get(clan_member_entity) {
            if let Some(online_member_game_client) = online_member.game_client {
                online_member_game_client
                    .server_message_tx
                    .send(ServerMessage::ClanUpdateInfo {
                        id: clan.unique_id,
                        mark: clan.mark,
                        level: clan.level,
                        points: clan.points,
                        money: clan.money,
                        skills: clan.skills.clone(),
                    })
                    .ok();
            }
        }
    }
}

pub fn clan_system(
    mut commands: Commands,
    mut clan_events: EventReader<ClanEvent>,
    query_member_connected: Query<MemberQuery, Changed<ClanMembership>>,
    query_member: Query<MemberQuery>,
    mut query_creator: Query<CreatorQuery>,
    mut query_clans: Query<&mut Clan>,
    mut server_messages: ResMut<ServerMessages>,
) {
    for event in clan_events.iter() {
        match event {
            ClanEvent::Create {
                creator: creator_entity,
                name,
                description,
                mark,
            } => {
                let Ok(mut creator) = query_creator.get_mut(*creator_entity) else {
                    continue;
                };

                // Cannot create a clan if already in one
                if creator.clan_membership.is_some() {
                    if let Some(game_client) = creator.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::ClanCreateError {
                                error: ClanCreateError::Failed,
                            })
                            .ok();
                    }
                    continue;
                }

                if creator.level.level < 30 {
                    if let Some(game_client) = creator.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::ClanCreateError {
                                error: ClanCreateError::UnmetCondition,
                            })
                            .ok();
                    }
                    continue;
                }

                if ClanStorage::exists(name) {
                    if let Some(game_client) = creator.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::ClanCreateError {
                                error: ClanCreateError::NameExists,
                            })
                            .ok();
                    }
                    continue;
                }

                let Ok(money) = creator.inventory.try_take_money(Money(1000000)) else {
                    if let Some(game_client) = creator.game_client {
                        game_client.server_message_tx.send(ServerMessage::ClanCreateError{ error: ClanCreateError::UnmetCondition }).ok();
                    }
                    continue;
                };

                let mut clan_storage = ClanStorage::new(name.clone(), description.clone(), *mark);
                clan_storage.members.push(ClanStorageMember::new(
                    creator.character_info.name.clone(),
                    ClanMemberPosition::Master,
                ));
                if clan_storage.try_create().is_err() {
                    if let Some(game_client) = creator.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::ClanCreateError {
                                error: ClanCreateError::Failed,
                            })
                            .ok();
                    }

                    creator.inventory.try_add_money(money).ok();
                    continue;
                }

                // Create clan entity
                let unique_id =
                    ClanUniqueId::new(QuestTriggerHash::from(name.as_str()).hash).unwrap();
                let members = vec![ClanMember::Online {
                    entity: *creator_entity,
                    position: ClanMemberPosition::Master,
                    contribution: ClanPoints(0),
                }];
                let clan_entity = commands
                    .spawn(Clan {
                        unique_id,
                        name: clan_storage.name.clone(),
                        description: clan_storage.description,
                        mark: clan_storage.mark,
                        money: clan_storage.money,
                        points: clan_storage.points,
                        level: clan_storage.level,
                        skills: clan_storage.skills,
                        members,
                    })
                    .id();

                // Add clan membership to creator
                commands
                    .entity(*creator_entity)
                    .insert(ClanMembership(Some(clan_entity)));

                // Update clan to nearby entities
                server_messages.send_entity_message(
                    creator.client_entity,
                    ServerMessage::CharacterUpdateClan {
                        client_entity_id: creator.client_entity.id,
                        id: unique_id,
                        mark: clan_storage.mark,
                        level: clan_storage.level,
                        name: clan_storage.name,
                        position: ClanMemberPosition::Master,
                    },
                );
            }
            &ClanEvent::MemberDisconnect {
                clan_entity,
                disconnect_entity,
                ref name,
                level,
                job,
            } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if let Some(clan_member) = clan.find_online_member_mut(disconnect_entity) {
                        let &mut ClanMember::Online { position, contribution, .. } = clan_member else { unreachable!() };
                        *clan_member = ClanMember::Offline {
                            name: name.clone(),
                            position,
                            contribution,
                            level,
                            job,
                        };

                        // Send message to other clan members that we have disconnected
                        for clan_member in clan.members.iter() {
                            let &ClanMember::Online { entity: clan_member_entity, .. } = clan_member else {
                                continue;
                            };

                            if let Ok(online_member) = query_member.get(clan_member_entity) {
                                if let Some(online_member_game_client) = online_member.game_client {
                                    online_member_game_client
                                        .server_message_tx
                                        .send(ServerMessage::ClanMemberDisconnected {
                                            name: name.clone(),
                                        })
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
            &ClanEvent::GetMemberList { entity } => {
                if let Ok(requestor) = query_member.get(entity) {
                    if let Some(clan) = requestor
                        .clan_membership
                        .and_then(|clan_entity| query_clans.get(clan_entity).ok())
                    {
                        let mut members = Vec::new();

                        for member in clan.members.iter() {
                            match *member {
                                ClanMember::Online {
                                    entity: member_entity,
                                    position,
                                    contribution,
                                } => {
                                    if let Ok(member) = query_member.get(member_entity) {
                                        members.push(ClanMemberInfo {
                                            name: member.character_info.name.clone(),
                                            position,
                                            contribution,
                                            channel_id: NonZeroUsize::new(1),
                                            level: *member.level,
                                            job: member.character_info.job,
                                        });
                                    }
                                }
                                ClanMember::Offline {
                                    ref name,
                                    position,
                                    contribution,
                                    level,
                                    job,
                                } => {
                                    members.push(ClanMemberInfo {
                                        name: name.clone(),
                                        position,
                                        contribution,
                                        channel_id: None,
                                        level,
                                        job,
                                    });
                                }
                            }
                        }

                        if let Some(game_client) = requestor.game_client {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::ClanMemberList { members })
                                .ok();
                        }
                    }
                }
            }
            &ClanEvent::AddLevel { clan_entity, level } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if let Some(level) = clan
                        .level
                        .0
                        .get()
                        .checked_add_signed(level)
                        .and_then(NonZeroU32::new)
                    {
                        clan.level = ClanLevel(level);
                        send_update_clan_info(&clan, &query_member);
                    }
                }
            }
            &ClanEvent::SetLevel { clan_entity, level } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    clan.level = level;
                    send_update_clan_info(&clan, &query_member);
                }
            }
            &ClanEvent::AddMoney { clan_entity, money } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if let Some(money) = clan.money.0.checked_add(money) {
                        clan.money = Money(money);
                        send_update_clan_info(&clan, &query_member);
                    }
                }
            }
            &ClanEvent::SetMoney { clan_entity, money } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    clan.money = money;
                    send_update_clan_info(&clan, &query_member);
                }
            }
            &ClanEvent::AddPoints {
                clan_entity,
                points,
            } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if let Some(points) = clan.points.0.checked_add_signed(points) {
                        clan.points = ClanPoints(points);
                        send_update_clan_info(&clan, &query_member);
                    }
                }
            }
            &ClanEvent::SetPoints {
                clan_entity,
                points,
            } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    clan.points = points;
                    send_update_clan_info(&clan, &query_member);
                }
            }
            &ClanEvent::AddSkill {
                clan_entity,
                skill_id,
            } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if !clan.skills.iter().any(|id| *id == skill_id) {
                        clan.skills.push(skill_id);
                        send_update_clan_info(&clan, &query_member);
                    }
                }
            }
            &ClanEvent::RemoveSkill {
                clan_entity,
                skill_id,
            } => {
                if let Ok(mut clan) = query_clans.get_mut(clan_entity) {
                    if clan.skills.iter().any(|id| *id == skill_id) {
                        clan.skills.retain(|id| *id != skill_id);
                        send_update_clan_info(&clan, &query_member);
                    }
                }
            }
        }
    }

    for connected_member in query_member_connected.iter() {
        let Some(clan) = connected_member.clan_membership.and_then(|clan_entity| query_clans.get(clan_entity).ok()) else {
            continue;
        };

        let Some(&ClanMember::Online { position: connected_member_position, contribution: connected_member_contribution, .. }) = clan.find_online_member(connected_member.entity) else { continue; };

        if let Some(game_client) = connected_member.game_client.as_ref() {
            game_client
                .server_message_tx
                .send(ServerMessage::ClanInfo {
                    id: clan.unique_id,
                    name: clan.name.clone(),
                    description: clan.description.clone(),
                    mark: clan.mark,
                    level: clan.level,
                    points: clan.points,
                    money: clan.money,
                    skills: clan.skills.clone(),
                    position: connected_member_position,
                    contribution: connected_member_contribution,
                })
                .ok();
        }

        // Send message to other clan members that we have connected
        for clan_member in clan.members.iter() {
            let &ClanMember::Online { entity: clan_member_entity, .. } = clan_member else {
                continue;
            };

            if clan_member_entity == connected_member.entity {
                continue;
            }

            if let Ok(online_member) = query_member.get(clan_member_entity) {
                if let Some(online_member_game_client) = online_member.game_client {
                    online_member_game_client
                        .server_message_tx
                        .send(ServerMessage::ClanMemberConnected {
                            name: connected_member.character_info.name.clone(),
                            channel_id: NonZeroUsize::new(1).unwrap(),
                        })
                        .ok();
                }
            }
        }
    }
}
