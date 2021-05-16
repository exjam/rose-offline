use legion::{component, system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore};

use crate::{
    data::{
        account::{AccountStorage, AccountStorageError},
        character::CharacterStorage,
    },
    game::{
        components::{Account, CharacterDeleteTime, CharacterList, ServerInfo, WorldClient},
        messages::client::{
            CharacterListItem, ClientMessage, ConnectionRequestError, ConnectionRequestResponse,
            CreateCharacter, CreateCharacterError, DeleteCharacterError, JoinServerResponse,
            SelectCharacterError,
        },
        resources::{GameData, LoginTokens},
    },
};

#[system(for_each)]
#[filter(!component::<Account>())]
pub fn world_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut WorldClient,
    #[resource] login_tokens: &mut LoginTokens,
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
                        match AccountStorage::try_load(&token.username, &message.password_md5) {
                            Ok(mut account) => {
                                // Load character list, deleting any characters ready for deletion
                                let mut character_list = CharacterList::new();
                                account.character_names.retain(|name| {
                                    CharacterStorage::try_load(name).map_or(false, |character| {
                                        if character
                                            .delete_time
                                            .as_ref()
                                            .map(|x| x.get_time_until_delete())
                                            .filter(|x| x.as_nanos() == 0)
                                            .is_some()
                                        {
                                            CharacterStorage::delete(&character.info.name).ok();
                                            false
                                        } else {
                                            character_list.characters.push(character);
                                            true
                                        }
                                    })
                                });

                                // Save account in case we have deleted characters
                                account.save().ok();
                                client.login_token = token.token;
                                client.selected_game_server = Some(token.selected_game_server);
                                cmd.add_component(*entity, Account::from(account));
                                cmd.add_component(*entity, character_list);
                                Ok(ConnectionRequestResponse {
                                    packet_sequence_id: 123,
                                })
                            }
                            Err(AccountStorageError::InvalidPassword) => {
                                Err(ConnectionRequestError::InvalidPassword)
                            }
                            Err(_) => Err(ConnectionRequestError::Failed),
                        }
                    });
                message.response_tx.send(response).ok();
            }
            _ => println!("Received unexpected client message"),
        }
    }
}

fn create_character(
    game_data: &GameData,
    message: &CreateCharacter,
) -> Result<CharacterStorage, CreateCharacterError> {
    let character = game_data
        .character_creator
        .create(
            message.name.clone(),
            message.gender,
            message.birth_stone,
            message.face,
            message.hair,
        )
        .map_err(|_| CreateCharacterError::InvalidValue)?;
    character
        .try_create()
        .map_err(|_| CreateCharacterError::Failed)?;
    Ok(character)
}

#[system(for_each)]
#[read_component(ServerInfo)]
pub fn world_server(
    world: &SubWorld,
    account: &mut Account,
    character_list: &mut CharacterList,
    client: &mut WorldClient,
    #[resource] login_tokens: &mut LoginTokens,
    #[resource] game_data: &GameData,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::GetCharacterList(message) => {
                let mut characters = Vec::new();
                for character in &character_list.characters {
                    characters.push(CharacterListItem::from(character));
                }
                message.response_tx.send(characters).ok();
            }
            ClientMessage::CreateCharacter(message) => {
                let response = if account.character_names.len() >= 5 {
                    Err(CreateCharacterError::NoMoreSlots)
                } else if message.name.len() < 4 || message.name.len() > 20 {
                    Err(CreateCharacterError::InvalidValue)
                } else if CharacterStorage::exists(&message.name) {
                    Err(CreateCharacterError::AlreadyExists)
                } else {
                    create_character(&game_data, &message)
                }
                .map(|character| {
                    let slot = account.character_names.len();
                    account.character_names.push(character.info.name.clone());
                    AccountStorage::from(&*account).save().ok();
                    character_list.characters.push(character);
                    slot as u8
                });
                message.response_tx.send(response).ok();
            }
            ClientMessage::DeleteCharacter(message) => {
                let response = character_list
                    .characters
                    .get_mut(message.slot as usize)
                    .filter(|character| character.info.name == message.name)
                    .map_or(Err(DeleteCharacterError::Failed), |character| {
                        if message.is_delete {
                            if character.delete_time.is_none() {
                                character.delete_time = Some(CharacterDeleteTime::new());
                            }
                        } else {
                            character.delete_time = None;
                        }
                        character.save().ok();
                        Ok(character.delete_time.clone())
                    });
                message.response_tx.send(response).ok();
            }
            ClientMessage::SelectCharacter(message) => {
                let response = character_list
                    .characters
                    .get_mut(message.slot as usize)
                    .filter(|character| character.info.name == message.name)
                    .map_or(Err(SelectCharacterError::Failed), |selected_character| {
                        // Set the selected_character for the login token
                        if let Some(token) = login_tokens
                            .tokens
                            .iter_mut()
                            .find(|t| t.token == client.login_token)
                        {
                            token.selected_character = selected_character.info.name.clone()
                        }

                        // Find the selected game server details
                        client
                            .selected_game_server
                            .and_then(|e| world.entry_ref(e).ok())
                            .map_or(Err(SelectCharacterError::Failed), |e| {
                                e.get_component::<ServerInfo>().map_or(
                                    Err(SelectCharacterError::Failed),
                                    |server_info| {
                                        Ok(JoinServerResponse {
                                            login_token: client.login_token,
                                            packet_codec_seed: server_info.packet_codec_seed,
                                            ip: server_info.ip.clone(),
                                            port: server_info.port,
                                        })
                                    },
                                )
                            })
                    });
                message.response_tx.send(response).ok();
            }
            _ => println!("Received unimplemented client message"),
        }
    }
}
