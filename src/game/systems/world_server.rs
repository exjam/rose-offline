use std::char;

use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use crate::game::data::character::{CharacterStorage, CharacterStorageError};
use crate::game::{
    components::CharacterListItem,
    messages::client::{
        ClientMessage, ConnectionRequestError, ConnectionRequestResponse, GetChannelListError,
        JoinServerError, JoinServerResponse, LoginError,
    },
};
use crate::game::{
    components::{Account, CharacterList, WorldClient},
    resources::LoginTokens,
    resources::ServerList,
};
use crate::game::{
    data::account::{AccountStorage, AccountStorageError},
    messages::client::CreateCharacterError,
};

#[system(for_each)]
#[write_component(Account)]
#[write_component(CharacterList)]
pub fn world_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut WorldClient,
    #[resource] login_tokens: &mut LoginTokens,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::ConnectionRequest(message) => {
                let response = if let Some((login_token, password_md5)) = message.login_token {
                    if let Some(username) = login_tokens
                        .tokens
                        .iter()
                        .find(|t| t.token == login_token)
                        .and_then(|t| Some(&t.username))
                    {
                        match AccountStorage::try_load(username, &password_md5) {
                            Ok(mut account) => {
                                // Load character list, processing any characters ready for deletion
                                let mut character_list = CharacterList::new();
                                account.character_names.retain(|name| {
                                    match CharacterStorage::try_load(name) {
                                        Ok(character) => {
                                            if character
                                                .delete_time
                                                .as_ref()
                                                .and_then(|x| Some(x.get_time_until_delete()))
                                                .filter(|x| x.as_nanos() == 0)
                                                .is_some()
                                            {
                                                CharacterStorage::delete(&character.info.name);
                                                false
                                            } else {
                                                character_list
                                                    .characters
                                                    .push(CharacterListItem::from(character));
                                                true
                                            }
                                        }
                                        Err(_) => false,
                                    }
                                });

                                // Save account in case we have deleted characters
                                account.save();

                                cmd.add_component(*entity, Account::from(account));
                                cmd.add_component(*entity, character_list);
                                Ok(ConnectionRequestResponse {
                                    packet_sequence_id: 123,
                                })
                            }
                            Err(error) => Err(match error {
                                AccountStorageError::InvalidPassword => {
                                    ConnectionRequestError::InvalidPassword
                                }
                                _ => ConnectionRequestError::Failed,
                            }),
                        }
                    } else {
                        Err(ConnectionRequestError::InvalidId)
                    }
                } else {
                    Err(ConnectionRequestError::Failed)
                };

                message.response_tx.send(response).ok();
            }
            _ => {
                client.pending_messages.push_back(message);
            }
        }
    }
}

#[system(for_each)]
pub fn world_server(
    account: &mut Account,
    character_list: &mut CharacterList,
    client: &mut WorldClient,
) {
    while let Some(message) = client.pending_messages.pop_front() {
        match message {
            ClientMessage::GetCharacterList(message) => {
                message.response_tx.send(character_list.clone()).ok();
            }
            ClientMessage::CreateCharacter(message) => {
                let response = if account.character_names.len() >= 5 {
                    Err(CreateCharacterError::NoMoreSlots)
                } else if message.name.len() < 4 || message.name.len() > 20 {
                    Err(CreateCharacterError::InvalidValue)
                } else if CharacterStorage::exists(&message.name) {
                    Err(CreateCharacterError::AlreadyExists)
                } else {
                    match CharacterStorage::try_create(
                        message.name,
                        message.gender,
                        message.birth_stone,
                        message.face,
                        message.hair,
                    ) {
                        Ok(character) => Ok(character),
                        Err(_) => Err(CreateCharacterError::Failed),
                    }
                }
                .and_then(|storage| {
                    let slot = account.character_names.len();
                    account.character_names.push(storage.info.name.clone());
                    AccountStorage::from(&*account).save().ok();
                    character_list
                        .characters
                        .push(CharacterListItem::from(storage));
                    Ok(slot as u8)
                });
                message.response_tx.send(response).ok();
            }
            _ => {
                panic!("Unhandled client message for world server!");
            }
        }
    }
}
