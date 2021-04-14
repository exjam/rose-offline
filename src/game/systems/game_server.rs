use legion::systems::CommandBuffer;
use legion::*;

use crate::game::components::{ClientEntityId, Destination, GameClient, Level, Position, Target};
use crate::game::data::{account::AccountStorage, character::CharacterStorage};
use crate::game::messages::client::{
    ClientMessage, ConnectionRequestError, GameConnectionResponse, JoinZoneResponse,
};
use crate::game::messages::server;
use crate::game::messages::server::ServerMessage;
use crate::game::resources::{ClientEntityIdList, LoginTokens, ServerMessages, ZoneEntityId};

#[system(for_each)]
pub fn game_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    #[resource] login_tokens: &mut LoginTokens,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::GameConnectionRequest(message) => {
                let response = login_tokens
                    .tokens
                    .iter()
                    .find(|t| t.token == message.login_token)
                    .ok_or(ConnectionRequestError::InvalidToken)
                    .and_then(|token| {
                        AccountStorage::try_load(&token.username, &message.password_md5)
                            .ok()
                            .ok_or(ConnectionRequestError::InvalidPassword)
                            .and_then(|_| {
                                CharacterStorage::try_load(&token.selected_character)
                                    .ok()
                                    .ok_or(ConnectionRequestError::Failed)
                            })
                            .and_then(|character| {
                                cmd.add_component(*entity, character.basic_stats.clone());
                                cmd.add_component(*entity, character.info.clone());
                                cmd.add_component(*entity, character.equipment.clone());
                                cmd.add_component(*entity, character.inventory.clone());
                                cmd.add_component(*entity, character.level.clone());
                                cmd.add_component(*entity, character.position.clone());

                                Ok(GameConnectionResponse {
                                    packet_sequence_id: 123,
                                    character_info: character.info,
                                    position: character.position,
                                    equipment: character.equipment,
                                    basic_stats: character.basic_stats,
                                    level: character.level,
                                    inventory: character.inventory,
                                })
                            })
                    });
                message.response_tx.send(response).ok();
            }
            _ => {
                client.pending_messages.push_back(message);
            }
        }
    }
}

#[system(for_each)]
pub fn game_server_join(
    cmd: &mut CommandBuffer,
    client: &mut GameClient,
    entity: &Entity,
    level: &Level,
    position: &Position,
    #[resource] client_entity_id_list: &mut ClientEntityIdList,
) {
    for message in client.pending_messages.iter_mut() {
        match message {
            ClientMessage::JoinZoneRequest(message) => {
                let entity_id = client_entity_id_list
                    .get_zone_mut(position.zone as usize)
                    .allocate(*entity)
                    .unwrap();

                cmd.add_component(*entity, ClientEntityId { id: entity_id });

                message
                    .response_tx
                    .take()
                    .unwrap()
                    .send(JoinZoneResponse {
                        entity_id: entity_id.0,
                        level: level.clone(),
                    })
                    .ok();
            }
            _ => (),
        }
    }
}

#[system(for_each)]
pub fn game_server_move(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    entity_id: &ClientEntityId,
    position: &Position,
    #[resource] client_entity_id_list: &mut ClientEntityIdList,
    #[resource] server_messages: &mut ServerMessages,
) {
    for message in client.pending_messages.iter_mut() {
        match message {
            ClientMessage::Move(message) => {
                let mut target_entity_id = 0;
                if message.target_entity_id > 0 {
                    if let Some(target_entity) = client_entity_id_list
                        .get_zone(position.zone as usize)
                        .get_entity(ZoneEntityId(message.target_entity_id))
                    {
                        target_entity_id = message.target_entity_id;
                        cmd.add_component(
                            *entity,
                            Target {
                                entity: target_entity,
                            },
                        );
                    } else {
                        cmd.remove_component::<Target>(*entity);
                    }
                } else {
                    cmd.remove_component::<Target>(*entity);
                }

                cmd.add_component(
                    *entity,
                    Destination {
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    },
                );

                let dx = position.x - message.x;
                let dy = position.y - message.y;
                let dz = position.z as f32 - message.z as f32;
                let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                server_messages.send_nearby_message(
                    position.clone(),
                    ServerMessage::MoveEntity(server::MoveEntity {
                        entity_id: entity_id.id.0,
                        target_entity_id: target_entity_id,
                        distance: distance as u16,
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    }),
                );
            }
            _ => (),
        }
    }
}
