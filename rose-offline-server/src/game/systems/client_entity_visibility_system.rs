use bevy::{
    ecs::{
        prelude::{Query, Res, ResMut},
        query::WorldQuery,
    },
    time::Time,
};

use rose_data::ClanMemberPosition;
use rose_game_common::messages::server::CharacterClanMembership;

use crate::game::{
    components::{
        AbilityValues, CharacterInfo, Clan, ClanMembership, ClientEntity, ClientEntityId,
        ClientEntitySector, ClientEntityType, ClientEntityVisibility, Command,
        CommandCastSkillTarget, CommandData, EntityExpireTime, Equipment, GameClient, HealthPoints,
        ItemDrop, Level, MoveMode, MoveSpeed, Npc, NpcStandingDirection, Owner, PersonalStore,
        Position, StatusEffects, Team,
    },
    messages::server::{ServerMessage, SpawnCommandState, SpawnEntityCharacter},
    resources::ClientEntityList,
};

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct GameClientQuery<'w> {
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    client_entity_visibility: &'w mut ClientEntityVisibility,
    game_client: &'w GameClient,
    position: &'w Position,
}

#[derive(WorldQuery)]
pub struct CharacterQuery<'w> {
    ability_values: &'w AbilityValues,
    character_info: &'w CharacterInfo,
    client_entity: &'w ClientEntity,
    command: &'w Command,
    equipment: &'w Equipment,
    health_points: &'w HealthPoints,
    level: &'w Level,
    move_mode: &'w MoveMode,
    move_speed: &'w MoveSpeed,
    position: &'w Position,
    status_effects: &'w StatusEffects,
    team: &'w Team,
    personal_store: Option<&'w PersonalStore>,
    clan_membership: &'w ClanMembership,
}

#[derive(WorldQuery)]
pub struct ItemDropQuery<'w> {
    item_drop: &'w ItemDrop,
    position: &'w Position,
    expire_time: &'w EntityExpireTime,
    owner: Option<&'w Owner>,
}

#[derive(WorldQuery)]
pub struct MonsterQuery<'w> {
    npc: &'w Npc,
    position: &'w Position,
    team: &'w Team,
    health: &'w HealthPoints,
    command: &'w Command,
    move_mode: &'w MoveMode,
    status_effects: &'w StatusEffects,
}

#[derive(WorldQuery)]
pub struct NpcQuery<'w> {
    npc: &'w Npc,
    direction: &'w NpcStandingDirection,
    position: &'w Position,
    team: &'w Team,
    health: &'w HealthPoints,
    command: &'w Command,
    move_mode: &'w MoveMode,
    status_effects: &'w StatusEffects,
}

#[derive(WorldQuery)]
pub struct TargetQuery<'w> {
    client_entity: &'w ClientEntity,
    position: &'w Position,
}

fn spawn_command_state(command: &Command, query_target: &Query<TargetQuery>) -> SpawnCommandState {
    match command.command {
        CommandData::Die { .. } => SpawnCommandState::Die,
        CommandData::Stop { .. } | CommandData::Standing => SpawnCommandState::Stop,
        CommandData::Move {
            destination,
            target: target_entity,
            ..
        } => {
            if let Some(target) =
                target_entity.and_then(|target_entity| query_target.get(target_entity).ok())
            {
                SpawnCommandState::Move {
                    target_position: target.position.position,
                    target_entity_id: Some(target.client_entity.id),
                }
            } else {
                SpawnCommandState::Move {
                    target_position: destination,
                    target_entity_id: None,
                }
            }
        }
        CommandData::Attack {
            target: target_entity,
        } => {
            if let Ok(target) = query_target.get(target_entity) {
                SpawnCommandState::Attack {
                    target_entity_id: target.client_entity.id,
                    target_position: target.position.position,
                }
            } else {
                SpawnCommandState::Stop
            }
        }
        CommandData::PickupItemDrop {
            target: target_entity,
        } => {
            if let Ok(target) = query_target.get(target_entity) {
                SpawnCommandState::PickupItemDrop {
                    target_entity_id: target.client_entity.id,
                    target_position: target.position.position,
                }
            } else {
                SpawnCommandState::Stop
            }
        }
        CommandData::PersonalStore => SpawnCommandState::PersonalStore,
        CommandData::CastSkill {
            skill_target: None, ..
        } => SpawnCommandState::CastSkillSelf,
        CommandData::CastSkill {
            skill_target: Some(CommandCastSkillTarget::Entity(_)),
            ..
        } => SpawnCommandState::CastSkillTargetEntity,
        CommandData::CastSkill {
            skill_target: Some(CommandCastSkillTarget::Position(_)),
            ..
        } => SpawnCommandState::CastSkillTargetPosition,
        CommandData::Sit | CommandData::Sitting => SpawnCommandState::Sit,
        CommandData::Emote { .. } => SpawnCommandState::Emote,
    }
}

pub fn client_entity_visibility_system(
    mut game_clients_query: Query<GameClientQuery>,
    entity_id_query: Query<&ClientEntity>,
    characters_query: Query<CharacterQuery>,
    item_drop_query: Query<ItemDropQuery>,
    monsters_query: Query<MonsterQuery>,
    npcs_query: Query<NpcQuery>,
    clan_query: Query<&Clan>,
    query_target: Query<TargetQuery>,
    mut client_entity_list: ResMut<ClientEntityList>,
    time: Res<Time>,
) {
    // First loop through all client entities and generate visibility changes that need to be sent
    for mut game_client in game_clients_query.iter_mut() {
        if let Some(client_entity_zone) = client_entity_list.get_zone(game_client.position.zone_id)
        {
            let sector_visible_entities = client_entity_zone
                .get_sector_visible_entities(game_client.client_entity_sector.sector);

            let mut visibility_difference =
                game_client.client_entity_visibility.entities ^ *sector_visible_entities;

            // Ignore self
            visibility_difference.set(game_client.client_entity.id.0, false);

            let mut remove_entity_ids = Vec::new();
            for index in visibility_difference.iter_ones() {
                let is_visible = sector_visible_entities.get(index).map_or(false, |b| *b);

                if !is_visible {
                    remove_entity_ids.push(ClientEntityId(index));
                } else if let Some((spawn_entity, spawn_client_entity, _)) =
                    client_entity_zone.get_entity(ClientEntityId(index))
                {
                    match spawn_client_entity.entity_type {
                        ClientEntityType::Character => {
                            if let Ok(character) = characters_query.get(*spawn_entity) {
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::SpawnEntityCharacter {
                                        data: Box::new(SpawnEntityCharacter {
                                            entity_id: spawn_client_entity.id,
                                            character_info: character.character_info.clone(),
                                            position: character.position.position,
                                            health: *character.health_points,
                                            team: character.team.clone(),
                                            equipment: character.equipment.clone(),
                                            level: *character.level,
                                            move_mode: *character.move_mode,
                                            move_speed: *character.move_speed,
                                            passive_attack_speed: character
                                                .ability_values
                                                .passive_attack_speed,
                                            status_effects: character.status_effects.active.clone(),
                                            spawn_command_state: spawn_command_state(
                                                character.command,
                                                &query_target,
                                            ),
                                            personal_store_info: character.personal_store.map(
                                                |personal_store| {
                                                    (
                                                        personal_store.skin,
                                                        personal_store.title.clone(),
                                                    )
                                                },
                                            ),
                                            clan_membership: character.clan_membership.and_then(
                                                |clan_entity| {
                                                    if let Ok(clan) = clan_query.get(clan_entity) {
                                                        Some(CharacterClanMembership {
                                                            clan_unique_id: clan.unique_id,
                                                            mark: clan.mark,
                                                            level: clan.level,
                                                            name: clan.name.clone(),
                                                            position: clan
                                                                .find_online_member(*spawn_entity)
                                                                .map_or(
                                                                    ClanMemberPosition::Junior,
                                                                    |member| member.position(),
                                                                ),
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                },
                                            ),
                                        }),
                                    })
                                    .ok();
                            }
                        }
                        ClientEntityType::ItemDrop => {
                            if let Ok(item_drop) = item_drop_query.get(*spawn_entity) {
                                if let Some(dropped_item) = item_drop.item_drop.item.clone() {
                                    let owner_entity_id = item_drop
                                        .owner
                                        .and_then(|owner| entity_id_query.get(owner.entity).ok())
                                        .map(|owner_client_entity| owner_client_entity.id);

                                    game_client
                                        .game_client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityItemDrop {
                                            entity_id: spawn_client_entity.id,
                                            dropped_item,
                                            position: item_drop.position.position,
                                            remaining_time: item_drop.expire_time.when
                                                - time.last_update().unwrap(),
                                            owner_entity_id,
                                        })
                                        .ok();
                                }
                            }
                        }
                        ClientEntityType::Monster => {
                            if let Ok(monster) = monsters_query.get(*spawn_entity) {
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::SpawnEntityMonster {
                                        entity_id: spawn_client_entity.id,
                                        npc: monster.npc.clone(),
                                        position: monster.position.position,
                                        team: monster.team.clone(),
                                        health: *monster.health,
                                        spawn_command_state: spawn_command_state(
                                            monster.command,
                                            &query_target,
                                        ),
                                        move_mode: *monster.move_mode,
                                        status_effects: monster.status_effects.active.clone(),
                                    })
                                    .ok();
                            }
                        }
                        ClientEntityType::Npc => {
                            if let Ok(npc) = npcs_query.get(*spawn_entity) {
                                game_client
                                    .game_client
                                    .server_message_tx
                                    .send(ServerMessage::SpawnEntityNpc {
                                        entity_id: spawn_client_entity.id,
                                        npc: npc.npc.clone(),
                                        direction: npc.direction.direction,
                                        position: npc.position.position,
                                        team: npc.team.clone(),
                                        health: *npc.health,
                                        spawn_command_state: spawn_command_state(
                                            npc.command,
                                            &query_target,
                                        ),
                                        move_mode: *npc.move_mode,
                                        status_effects: npc.status_effects.active.clone(),
                                    })
                                    .ok();
                            }
                        }
                    }
                }
            }

            if !remove_entity_ids.is_empty() {
                game_client
                    .game_client
                    .server_message_tx
                    .send(ServerMessage::RemoveEntities {
                        entity_ids: remove_entity_ids,
                    })
                    .ok();
            }

            // Update visibility
            game_client.client_entity_visibility.entities = *sector_visible_entities;
        }
    }

    client_entity_list.process_zone_leavers();
}
