use bevy_ecs::prelude::{Query, Res, ResMut};

use crate::game::{
    components::{
        AbilityValues, CharacterInfo, ClientEntity, ClientEntityId, ClientEntityType,
        ClientEntityVisibility, Command, Destination, Equipment, ExpireTime, GameClient,
        HealthPoints, ItemDrop, Level, MoveMode, MoveSpeed, Npc, NpcStandingDirection, Owner,
        PersonalStore, Position, StatusEffects, Target, Team,
    },
    messages::server::{
        RemoveEntities, ServerMessage, SpawnEntityCharacter, SpawnEntityItemDrop,
        SpawnEntityMonster, SpawnEntityNpc,
    },
    resources::{ClientEntityList, ServerTime},
};

pub fn client_entity_visibility_system(
    mut clients_query: Query<(
        &mut ClientEntityVisibility,
        &GameClient,
        &ClientEntity,
        &Position,
    )>,
    entity_id_query: Query<&ClientEntity>,
    characters_query: Query<(
        &AbilityValues,
        &CharacterInfo,
        &ClientEntity,
        &Command,
        &Equipment,
        &HealthPoints,
        &Level,
        &MoveMode,
        &MoveSpeed,
        &Position,
        &StatusEffects,
        &Team,
        Option<&Destination>,
        Option<&Target>,
        Option<&PersonalStore>,
    )>,
    item_drop_query: Query<(&ItemDrop, &Position, &ExpireTime, Option<&Owner>)>,
    monsters_query: Query<(
        &Npc,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        &MoveMode,
        &StatusEffects,
        Option<&Destination>,
        Option<&Target>,
    )>,
    npcs_query: Query<(
        &Npc,
        &NpcStandingDirection,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        &MoveMode,
        &StatusEffects,
        Option<&Destination>,
        Option<&Target>,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    // First loop through all client entities and generate visibility changes that need to be sent
    clients_query.for_each_mut(
        |(
            mut visibility,
            visibility_game_client,
            visibility_client_entity,
            visibility_position,
        )| {
            if let Some(client_entity_zone) =
                client_entity_list.get_zone(visibility_position.zone_id)
            {
                let sector_visible_entities =
                    client_entity_zone.get_sector_visible_entities(visibility_client_entity.sector);

                let mut visibility_difference = visibility.entities ^ *sector_visible_entities;

                // Ignore self
                visibility_difference.set(visibility_client_entity.id.0, false);

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
                                if let Ok((
                                    spawn_ability_values,
                                    spawn_character_info,
                                    spawn_client_entity,
                                    spawn_command,
                                    spawn_equipment,
                                    spawn_health_points,
                                    spawn_level,
                                    spawn_move_mode,
                                    spawn_move_speed,
                                    spawn_position,
                                    spawn_status_effects,
                                    spawn_team,
                                    spawn_destination,
                                    spawn_target,
                                    spawn_personal_store,
                                )) = characters_query.get(*spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query.get(spawn_target.entity).ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    visibility_game_client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityCharacter(Box::new(
                                            SpawnEntityCharacter {
                                                entity_id: spawn_client_entity.id,
                                                character_info: spawn_character_info.clone(),
                                                position: spawn_position.clone(),
                                                destination: spawn_destination.cloned(),
                                                health: *spawn_health_points,
                                                team: spawn_team.clone(),
                                                equipment: spawn_equipment.clone(),
                                                level: spawn_level.clone(),
                                                move_mode: *spawn_move_mode,
                                                move_speed: *spawn_move_speed,
                                                passive_attack_speed: spawn_ability_values
                                                    .passive_attack_speed,
                                                status_effects: spawn_status_effects.clone(),
                                                command: spawn_command.clone(),
                                                target_entity_id,
                                                personal_store_info: spawn_personal_store.map(
                                                    |personal_store| {
                                                        (
                                                            personal_store.skin,
                                                            personal_store.title.clone(),
                                                        )
                                                    },
                                                ),
                                            },
                                        )))
                                        .ok();
                                }
                            }
                            ClientEntityType::ItemDrop => {
                                if let Ok((
                                    spawn_item_drop,
                                    spawn_position,
                                    spawn_expire_time,
                                    spawn_owner,
                                )) = item_drop_query.get(*spawn_entity)
                                {
                                    if let Some(spawn_dropped_item) = spawn_item_drop.item.clone() {
                                        let owner_entity_id = spawn_owner
                                            .and_then(|spawn_owner| {
                                                entity_id_query.get(spawn_owner.entity).ok()
                                            })
                                            .map(|spawn_owner_client_entity| {
                                                spawn_owner_client_entity.id
                                            });

                                        visibility_game_client
                                            .server_message_tx
                                            .send(ServerMessage::SpawnEntityItemDrop(
                                                SpawnEntityItemDrop {
                                                    entity_id: spawn_client_entity.id,
                                                    dropped_item: spawn_dropped_item,
                                                    position: spawn_position.clone(),
                                                    remaining_time: spawn_expire_time.when
                                                        - server_time.now,
                                                    owner_entity_id,
                                                },
                                            ))
                                            .ok();
                                    }
                                }
                            }
                            ClientEntityType::Monster => {
                                if let Ok((
                                    spawn_npc,
                                    spawn_position,
                                    spawn_team,
                                    spawn_health,
                                    spawn_command,
                                    spawn_move_mode,
                                    spawn_status_effects,
                                    spawn_destination,
                                    spawn_target,
                                )) = monsters_query.get(*spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query.get(spawn_target.entity).ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    visibility_game_client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityMonster(
                                            SpawnEntityMonster {
                                                entity_id: spawn_client_entity.id,
                                                npc: spawn_npc.clone(),
                                                position: spawn_position.clone(),
                                                team: spawn_team.clone(),
                                                health: *spawn_health,
                                                destination: spawn_destination.cloned(),
                                                command: spawn_command.clone(),
                                                target_entity_id,
                                                move_mode: *spawn_move_mode,
                                                status_effects: spawn_status_effects.clone(),
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
                                    spawn_move_mode,
                                    spawn_status_effects,
                                    spawn_destination,
                                    spawn_target,
                                )) = npcs_query.get(*spawn_entity)
                                {
                                    let target_entity_id = spawn_target
                                        .and_then(|spawn_target| {
                                            entity_id_query.get(spawn_target.entity).ok()
                                        })
                                        .map(|spawn_target_client_entity| {
                                            spawn_target_client_entity.id
                                        });

                                    visibility_game_client
                                        .server_message_tx
                                        .send(ServerMessage::SpawnEntityNpc(SpawnEntityNpc {
                                            entity_id: spawn_client_entity.id,
                                            npc: spawn_npc.clone(),
                                            direction: spawn_direction.clone(),
                                            position: spawn_position.clone(),
                                            team: spawn_team.clone(),
                                            health: *spawn_health,
                                            destination: spawn_destination.cloned(),
                                            command: spawn_command.clone(),
                                            target_entity_id,
                                            move_mode: *spawn_move_mode,
                                            status_effects: spawn_status_effects.clone(),
                                        }))
                                        .ok();
                                }
                            }
                        }
                    }
                }

                if !remove_entity_ids.is_empty() {
                    visibility_game_client
                        .server_message_tx
                        .send(ServerMessage::RemoveEntities(RemoveEntities::new(
                            remove_entity_ids,
                        )))
                        .ok();
                }

                // Update visibility
                visibility.entities = *sector_visible_entities;
            }
        },
    );

    client_entity_list.process_zone_leavers();
}
