use std::{char, collections::VecDeque};

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::TryRead;
use legion::*;
use std::f32;

use crate::game::{
    components::{Account, CharacterList, GameClient, ServerInfo},
    messages::{client::JoinZoneResponse, server::ServerMessage},
    resources::LoginTokens,
    resources::{ServerList, ServerMessages},
};
use crate::game::{
    data::account::{AccountStorage, AccountStorageError},
    messages::client::CreateCharacterError,
};
use crate::game::{
    messages::{
        client::{
            ClientMessage, ConnectionRequestError, ConnectionRequestResponse, GetChannelListError,
            JoinServerError, JoinServerResponse, LoginError,
        },
        server,
    },
    resources::ClientEntityIdList,
};
use crate::{
    game::{
        components::CharacterDeleteTime,
        components::{
            BasicStats, CharacterInfo, ClientEntityId, Destination, Equipment, Inventory, Level,
            Position, Target,
        },
        data::character::{CharacterStorage, CharacterStorageError},
        messages::client::{
            CharacterListItem, DeleteCharacterError, GetInitialCharacterData, InitialCharacterData,
            SelectCharacterError,
        },
        resources::{LoginToken, ZoneEntityId},
    },
    protocol::Client,
};

#[system(for_each)]
pub fn game_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut GameClient,
    #[resource] login_tokens: &mut LoginTokens,
    #[resource] client_entity_id_list: &mut ClientEntityIdList,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::ConnectionRequest(message) => {
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
                                let entity_id = client_entity_id_list
                                    .get_zone_mut(character.position.zone as usize)
                                    .allocate(*entity)
                                    .unwrap();

                                cmd.add_component(*entity, character.basic_stats);
                                cmd.add_component(*entity, character.info);
                                cmd.add_component(*entity, character.equipment);
                                cmd.add_component(*entity, character.inventory);
                                cmd.add_component(*entity, character.level);
                                cmd.add_component(*entity, character.position);
                                cmd.add_component(*entity, ClientEntityId { id: entity_id });

                                Ok(ConnectionRequestResponse {
                                    packet_sequence_id: 123,
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

                let dx = (position.x - message.x);
                let dy = (position.y - message.y);
                let dz = (position.z as f32 - message.z as f32);
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

#[system(for_each)]
pub fn game_server(
    client: &mut GameClient,
    entity_id: &ClientEntityId,
    character_info: &CharacterInfo,
    position: &Position,
    equipment: &Equipment,
    basic_stats: &BasicStats,
    level: &Level,
    inventory: &Inventory,
) {
    for message in client.pending_messages.iter_mut() {
        match message {
            ClientMessage::GetInitialCharacterData(message) => {
                // TODO: This might be better merged into connect request response.
                message
                    .response_tx
                    .take()
                    .unwrap()
                    .send(InitialCharacterData {
                        character_info: character_info.clone(),
                        position: position.clone(),
                        equipment: equipment.clone(),
                        basic_stats: basic_stats.clone(),
                        level: level.clone(),
                        inventory: inventory.clone(),
                    })
                    .ok();
            }
            ClientMessage::JoinZoneRequest(message) => {
                // TODO: We probably need to wait until this message to assign ClientEntityId.
                message
                    .response_tx
                    .take()
                    .unwrap()
                    .send(JoinZoneResponse {
                        entity_id: entity_id.id.0,
                        level: level.clone(),
                    })
                    .ok();
            }
            _ => (),
        }
    }
}
