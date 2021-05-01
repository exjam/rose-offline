use legion::{system, world::SubWorld, Entity, Query};
use std::collections::HashSet;
use tokio::sync::mpsc::UnboundedSender;

use crate::game::{
    components::{
        ClientEntity, ClientEntityVisibility, GameClient, Npc, NpcStandingDirection, Position, Team,
    },
    messages::server::{RemoveEntities, ServerMessage, SpawnEntityMonster, SpawnEntityNpc},
    resources::ClientEntityList,
};

pub struct VisibilityChange {
    client: UnboundedSender<ServerMessage>,
    remove_entities: HashSet<Entity>,
    spawn_entities: HashSet<Entity>,
}

#[system]
pub fn client_entity_visibility(
    world: &mut SubWorld,
    clients: &mut Query<(
        Entity,
        &GameClient,
        &ClientEntity,
        &mut ClientEntityVisibility,
        &Position,
    )>,
    entity_id_query: &mut Query<&ClientEntity>,
    npcs_query: &mut Query<(&ClientEntity, &Npc, &NpcStandingDirection, &Position, &Team)>,
    monsters_query: &mut Query<(&ClientEntity, &Npc, &Position, &Team)>,
    #[resource] client_entity_list: &ClientEntityList,
) {
    let mut visibility_changes = Vec::new();

    // First loop through all client entities and generate visibility changes that need to be sent
    clients.for_each_mut(
        world,
        |(entity, client, client_entity, client_visibility, position)| {
            if let Some(zone) = client_entity_list.get_zone(position.zone as usize) {
                let sector = zone.get_sector(client_entity.sector);

                let mut remove_entities = &client_visibility.entities - &sector.entities;
                let mut spawn_entities = &sector.entities - &client_visibility.entities;

                // Ignore self in entity lists
                remove_entities.remove(entity);
                spawn_entities.remove(entity);

                if !remove_entities.is_empty() || !spawn_entities.is_empty() {
                    visibility_changes.push(VisibilityChange {
                        client: client.server_message_tx.clone(),
                        remove_entities,
                        spawn_entities,
                    });
                }

                client_visibility.entities = sector.entities.clone();
            }
        },
    );

    // Now process the visibility changes into server messages
    for visibility_change in visibility_changes.into_iter() {
        // Collect list of client entity id we should remove
        if !visibility_change.remove_entities.is_empty() {
            let remove_entities: Vec<u16> = visibility_change
                .remove_entities
                .iter()
                .map(|entity| {
                    entity_id_query
                        .get(world, *entity)
                        .ok()
                        .map(|client_entity| client_entity.id.0)
                })
                .filter_map(|x| x)
                .collect();

            if !remove_entities.is_empty() {
                visibility_change
                    .client
                    .send(ServerMessage::RemoveEntities(RemoveEntities::new(
                        remove_entities,
                    )))
                    .ok();
            }
        }

        // Now send spawn messages
        if !visibility_change.spawn_entities.is_empty() {
            for entity in visibility_change.spawn_entities.iter() {
                // TODO: Try read the entity as a character
                // TODO: Try read the entity as a dropped item

                // Try read the entity as an NPC
                if npcs_query
                    .get(world, *entity)
                    .map(|(client_entity, npc, direction, position, team)| {
                        visibility_change
                            .client
                            .send(ServerMessage::SpawnEntityNpc(SpawnEntityNpc {
                                entity_id: client_entity.id.0,
                                npc: npc.clone(),
                                direction: direction.clone(),
                                position: position.clone(),
                                team: team.clone(),
                            }))
                            .ok();
                        ()
                    })
                    .ok()
                    .is_some()
                {
                    continue;
                }

                // Try read the entity as a monster
                if monsters_query
                    .get(world, *entity)
                    .map(|(client_entity, npc, position, team)| {
                        visibility_change
                            .client
                            .send(ServerMessage::SpawnEntityMonster(SpawnEntityMonster {
                                entity_id: client_entity.id.0,
                                npc: npc.clone(),
                                position: position.clone(),
                                team: team.clone(),
                            }))
                            .ok();
                        ()
                    })
                    .ok()
                    .is_some()
                {
                    continue;
                }
            }
        }
    }
}
