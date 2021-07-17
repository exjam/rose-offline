use legion::{system, world::SubWorld, Entity, Query};

use crate::game::{
    components::{
        AbilityValues, CharacterInfo, ClientEntity, ClientEntityType, ClientEntityVisibility,
        Command, Destination, DroppedItem, Equipment, ExpireTime, GameClient, HealthPoints, Level,
        Npc, NpcStandingDirection, Owner, Position, Target, Team,
    },
    messages::server::{
        RemoveEntities, ServerMessage, SpawnEntityCharacter, SpawnEntityDroppedItem,
        SpawnEntityMonster, SpawnEntityNpc,
    },
    resources::{ClientEntityList, ServerTime},
};

#[allow(clippy::type_complexity)]
#[system]
pub fn client_entity_visibility(
    world: &mut SubWorld,
    clients_query: &mut Query<(
        Entity,
        &GameClient,
        &ClientEntity,
        &mut ClientEntityVisibility,
        &Position,
    )>,
    entity_id_query: &mut Query<&ClientEntity>,
    characters_query: &mut Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &Command,
        &Equipment,
        &HealthPoints,
        &Level,
        &Position,
        &Team,
        Option<&Destination>,
        Option<&Target>,
    )>,
    dropped_item_query: &mut Query<(&Option<DroppedItem>, &Position, &ExpireTime, Option<&Owner>)>,
    monsters_query: &mut Query<(
        &Npc,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        Option<&Destination>,
        Option<&Target>,
    )>,
    npcs_query: &mut Query<(
        &Npc,
        &NpcStandingDirection,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        Option<&Destination>,
        Option<&Target>,
    )>,
    #[resource] client_entity_list: &ClientEntityList,
    #[resource] server_time: &ServerTime,
) {
    let (mut clients_query_world, mut world) = world.split_for_query(clients_query);
    let (entity_id_query_world, mut world) = world.split_for_query(entity_id_query);
    let (characters_query_world, mut world) = world.split_for_query(characters_query);
    let (dropped_item_query_world, mut world) = world.split_for_query(dropped_item_query);
    let (monster_query_world, mut world) = world.split_for_query(monsters_query);
    let (npc_query_world, mut _world) = world.split_for_query(npcs_query);

    // First loop through all client entities and generate visibility changes that need to be sent
    clients_query.for_each_mut(
        &mut clients_query_world,
        |(entity, client, client_entity, client_visibility, position)| {
            if let Some(zone) = client_entity_list.get_zone(position.zone as usize) {
                let sector_visible_entities =
                    zone.get_sector_visible_entities(client_entity.sector);

                let mut remove_entities = &client_visibility.entities - sector_visible_entities;
                let mut spawn_entities = sector_visible_entities - &client_visibility.entities;

                // Ignore self in entity lists
                remove_entities.remove(entity);
                spawn_entities.remove(entity);

                if remove_entities.is_empty() && spawn_entities.is_empty() {
                    return;
                }

                client_visibility.entities = sector_visible_entities.clone();

                // Send remove entity message
                if !remove_entities.is_empty() {
                    client
                        .server_message_tx
                        .send(ServerMessage::RemoveEntities(RemoveEntities::new(
                            remove_entities
                                .iter()
                                .filter_map(|remove_entity| {
                                    entity_id_query
                                        .get(&entity_id_query_world, *remove_entity)
                                        .ok()
                                })
                                .map(|remove_client_entity| remove_client_entity.id)
                                .collect(),
                        )))
                        .ok();
                }

                // Send spawn entity messages
                for spawn_entity in spawn_entities {
                    if let Ok(spawn_client_entity) =
                        entity_id_query.get(&entity_id_query_world, spawn_entity)
                    {
                        match spawn_client_entity.entity_type {
                            ClientEntityType::Character => {
                                if let Ok((
                                    spawn_ability_values,
                                    spawn_character_info,
                                    spawn_client_entity,
                                    spawn_command,
                                    spawn_equipment,
                                    spawn_health_points,
                                    spawn_level,
                                    spawn_position,
                                    spawn_team,
                                    spawn_destination,
                                    spawn_target,
                                )) = characters_query.get(&characters_query_world, spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query
                                                .get(&entity_id_query_world, spawn_target.entity)
                                                .ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityCharacter(Box::new(
                                            SpawnEntityCharacter {
                                                entity_id: spawn_client_entity.id,
                                                character_info: spawn_character_info.clone(),
                                                position: spawn_position.clone(),
                                                destination: spawn_destination.cloned(),
                                                health: spawn_health_points.clone(),
                                                team: spawn_team.clone(),
                                                equipment: spawn_equipment.clone(),
                                                level: spawn_level.clone(),
                                                run_speed: spawn_ability_values.run_speed,
                                                passive_attack_speed: spawn_ability_values
                                                    .passive_attack_speed,
                                                command: spawn_command.clone(),
                                                target_entity_id,
                                            },
                                        )))
                                        .ok();
                                }
                            }
                            ClientEntityType::DroppedItem => {
                                if let Ok((
                                    Some(spawn_item),
                                    spawn_position,
                                    spawn_expire_time,
                                    spawn_owner,
                                )) =
                                    dropped_item_query.get(&dropped_item_query_world, spawn_entity)
                                {
                                    let owner_entity_id = spawn_owner
                                        .and_then(|spawn_owner| {
                                            entity_id_query
                                                .get(&entity_id_query_world, spawn_owner.entity)
                                                .ok()
                                        })
                                        .map(|spawn_owner_client_entity| {
                                            spawn_owner_client_entity.id
                                        });

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityDroppedItem(
                                            SpawnEntityDroppedItem {
                                                entity_id: spawn_client_entity.id,
                                                dropped_item: spawn_item.clone(),
                                                position: spawn_position.clone(),
                                                remaining_time: spawn_expire_time.when
                                                    - server_time.now,
                                                owner_entity_id,
                                            },
                                        ))
                                        .ok();
                                }
                            }
                            ClientEntityType::Monster => {
                                if let Ok((
                                    spawn_npc,
                                    spawn_position,
                                    spawn_team,
                                    spawn_health,
                                    spawn_command,
                                    spawn_destination,
                                    spawn_target,
                                )) = monsters_query.get(&monster_query_world, spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query
                                                .get(&entity_id_query_world, spawn_target.entity)
                                                .ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityMonster(
                                            SpawnEntityMonster {
                                                entity_id: spawn_client_entity.id,
                                                npc: spawn_npc.clone(),
                                                position: spawn_position.clone(),
                                                team: spawn_team.clone(),
                                                health: spawn_health.clone(),
                                                destination: spawn_destination.cloned(),
                                                command: spawn_command.clone(),
                                                target_entity_id,
                                            },
                                        ))
                                        .ok();
                                }
                            }
                            ClientEntityType::Npc => {
                                if let Ok((
                                    spawn_npc,
                                    spawn_direction,
                                    spawn_position,
                                    spawn_team,
                                    spawn_health,
                                    spawn_command,
                                    spawn_destination,
                                    spawn_target,
                                )) = npcs_query.get(&npc_query_world, spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query
                                                .get(&entity_id_query_world, spawn_target.entity)
                                                .ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityNpc(SpawnEntityNpc {
                                            entity_id: spawn_client_entity.id,
                                            npc: spawn_npc.clone(),
                                            direction: spawn_direction.clone(),
                                            position: spawn_position.clone(),
                                            team: spawn_team.clone(),
                                            health: spawn_health.clone(),
                                            destination: spawn_destination.cloned(),
                                            command: spawn_command.clone(),
                                            target_entity_id,
                                        }))
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        },
    );
}
