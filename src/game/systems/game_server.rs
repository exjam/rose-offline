use std::char;

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use crate::game::{components::CharacterDeleteTime, components::{BasicStats, CharacterInfo, ClientEntityId, Equipment, Inventory, Level, Position}, data::character::{CharacterStorage, CharacterStorageError}, messages::client::{CharacterListItem, DeleteCharacterError, GetInitialCharacterData, InitialCharacterData, SelectCharacterError}, resources::LoginToken};
use crate::game::{
    components::{Account, CharacterList, GameClient, ServerInfo},
    resources::LoginTokens,
    resources::ServerList,
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
                println!("GS: ClientMessage::ConnectionRequest");
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
                println!("GS: Pending message");
                client.pending_messages.push_back(message);
            }
        }
    }
}

#[system(for_each)]
pub fn game_server(
    client: &mut GameClient,
    character_info: &CharacterInfo,
    position: &Position,
    equipment: &Equipment,
    basic_stats: &BasicStats,
    level: &Level,
    inventory: &Inventory)
{
    while let Some(message) = client.pending_messages.pop_front() {
        match message {
            ClientMessage::GetInitialCharacterData(message) => {
                message.response_tx.send(InitialCharacterData {
                    character_info: character_info.clone(),
                    position: position.clone(),
                    equipment: equipment.clone(),
                    basic_stats: basic_stats.clone(),
                    level: level.clone(),
                    inventory: inventory.clone(),
                }).ok();
            }
            _ => ()
        }
    }
}
