use legion::{system, world::SubWorld, Entity, Query};

use crate::game::{
    components::{
        ClientEntity, ClientEntityType, ClientEntityVisibility, Command, Destination, GameClient,
        HealthPoints, Npc, NpcStandingDirection, Position, Team,
    },
    messages::server::{RemoveEntities, ServerMessage, SpawnEntityMonster, SpawnEntityNpc},
    resources::ClientEntityList,
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
    npcs_query: &mut Query<(
        &Npc,
        &NpcStandingDirection,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        Option<&Destination>,
    )>,
    monsters_query: &mut Query<(
        &Npc,
        &Position,
        &Team,
        &HealthPoints,
        &Command,
        Option<&Destination>,
    )>,
    #[resource] client_entity_list: &ClientEntityList,
) {
    let (mut clients_query_world, mut world) = world.split_for_query(clients_query);
    let (entity_id_query_world, mut world) = world.split_for_query(entity_id_query);
    let (npc_query_world, mut world) = world.split_for_query(npcs_query);
    let (monster_query_world, mut _world) = world.split_for_query(monsters_query);

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
                                todo!();
                            }
                            ClientEntityType::Monster => {
                                if let Ok((
                                    spawn_npc,
                                    spawn_position,
                                    spawn_team,
                                    spawn_health,
                                    spawn_command,
                                    spawn_destination,
                                )) = monsters_query.get(&monster_query_world, spawn_entity)
                                {
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
                                                target_entity_id: None, // TODO: Target entity id !
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
                                )) = npcs_query.get(&npc_query_world, spawn_entity)
                                {
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
                                            target_entity_id: None, // TODO: Target entity id !
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
