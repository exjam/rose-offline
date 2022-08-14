use bevy::ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use log::warn;

use rose_game_common::data::Password;

use crate::game::{
    components::{Account, CharacterDeleteTime, CharacterList, ServerInfo, WorldClient},
    messages::{
        client::{ClientMessage, CreateCharacter},
        server::{
            CharacterListItem, ConnectionRequestError, ConnectionResponse, CreateCharacterError,
            CreateCharacterResponse, DeleteCharacterError, DeleteCharacterResponse,
            JoinServerResponse, SelectCharacterError, ServerMessage,
        },
    },
    resources::{GameData, LoginTokens},
    storage::{
        account::{AccountStorage, AccountStorageError},
        character::CharacterStorage,
    },
};

fn handle_world_connection_request(
    commands: &mut Commands,
    login_tokens: &mut LoginTokens,
    entity: Entity,
    world_client: &mut WorldClient,
    token_id: u32,
    password: &Password,
) -> Result<ConnectionResponse, ConnectionRequestError> {
    let login_token = login_tokens
        .get_token_mut(token_id)
        .ok_or(ConnectionRequestError::InvalidToken)?;
    if login_token.world_client.is_some() || login_token.game_client.is_some() {
        return Err(ConnectionRequestError::InvalidToken);
    }

    let mut account =
        AccountStorage::try_load(&login_token.username, password).map_err(|error| {
            match error.downcast_ref::<AccountStorageError>() {
                Some(AccountStorageError::InvalidPassword) => {
                    ConnectionRequestError::InvalidPassword
                }
                _ => {
                    log::error!(
                        "Failed to load account {} with error {:?}",
                        &login_token.username,
                        error
                    );
                    ConnectionRequestError::Failed
                }
            }
        })?;

    // Load character list, deleting any characters ready for deletion
    let mut character_list = CharacterList::default();
    account
        .character_names
        .retain(|name| match CharacterStorage::try_load(name) {
            Ok(character) => {
                if character
                    .delete_time
                    .as_ref()
                    .map(|x| x.get_time_until_delete())
                    .filter(|x| x.as_nanos() == 0)
                    .is_some()
                {
                    match CharacterStorage::delete(&character.info.name) {
                        Ok(_) => log::error!(
                            "Deleted character {} as delete timer has expired.",
                            &character.info.name
                        ),
                        Err(error) => log::error!(
                            "Failed to delete character {} with error {:?}",
                            &character.info.name,
                            error
                        ),
                    }
                    false
                } else {
                    character_list.push(character);
                    true
                }
            }
            Err(error) => {
                log::error!("Failed to load character {} with error {:?}", name, error);
                false
            }
        });
    account.save().ok();

    // Update entity
    commands
        .entity(entity)
        .insert(Account::from(account))
        .insert(character_list);

    // Update token
    login_token.world_client = Some(entity);
    world_client.login_token = login_token.token;
    world_client.selected_game_server = Some(login_token.selected_game_server);

    Ok(ConnectionResponse {
        packet_sequence_id: 123,
    })
}

pub fn world_server_authentication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorldClient), Without<Account>>,
    mut login_tokens: ResMut<LoginTokens>,
) {
    query.for_each_mut(|(entity, mut world_client)| {
        if let Ok(message) = world_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::ConnectionRequest(message) => {
                    let response =
                        ServerMessage::ConnectionResponse(handle_world_connection_request(
                            &mut commands,
                            login_tokens.as_mut(),
                            entity,
                            world_client.as_mut(),
                            message.login_token,
                            &message.password,
                        ));
                    world_client.server_message_tx.send(response).ok();
                }
                _ => panic!("Received unexpected client message {:?}", message),
            }
        }
    });
}

fn create_character(
    game_data: &GameData,
    message: &CreateCharacter,
) -> Result<CharacterStorage, anyhow::Error> {
    let character = game_data
        .character_creator
        .create(
            message.name.clone(),
            message.gender,
            message.birth_stone as u8,
            message.face as u8,
            message.hair as u8,
        )
        .map_err(|_| CreateCharacterError::InvalidValue)?;

    character.try_create()?;

    Ok(character)
}

pub fn world_server_system(
    mut world_client_query: Query<(&mut WorldClient, &mut Account, &mut CharacterList)>,
    server_info_query: Query<&ServerInfo>,
    mut login_tokens: ResMut<LoginTokens>,
    game_data: Res<GameData>,
) {
    world_client_query.for_each_mut(|(world_client, mut account, mut character_list)| {
        if let Ok(message) = world_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GetCharacterList => {
                    let mut characters = Vec::new();
                    for character in character_list.iter() {
                        characters.push(CharacterListItem {
                            info: character.info.clone(),
                            level: character.level,
                            delete_time: character.delete_time.clone(),
                            equipment: character.equipment.clone(),
                        });
                    }
                    world_client
                        .server_message_tx
                        .send(ServerMessage::CharacterList(characters))
                        .ok();
                }
                ClientMessage::CreateCharacter(message) => {
                    let response = if account.character_names.len() >= 5 {
                        Err(CreateCharacterError::NoMoreSlots)
                    } else if message.name.len() < 4 || message.name.len() > 20 {
                        Err(CreateCharacterError::InvalidValue)
                    } else if CharacterStorage::exists(&message.name) {
                        Err(CreateCharacterError::AlreadyExists)
                    } else {
                        match create_character(&game_data, &message) {
                            Ok(character) => {
                                let character_slot = account.character_names.len();
                                account.character_names.push(character.info.name.clone());
                                AccountStorage::from(&*account).save().ok();
                                character_list.push(character);
                                Ok(CreateCharacterResponse { character_slot })
                            }
                            Err(error) => {
                                log::error!(
                                    "Failed to create character {} with error {:?}",
                                    &message.name,
                                    error
                                );
                                Err(error
                                    .downcast_ref::<CreateCharacterError>()
                                    .unwrap_or(&CreateCharacterError::Failed)
                                    .clone())
                            }
                        }
                    };

                    world_client
                        .server_message_tx
                        .send(ServerMessage::CreateCharacter(response))
                        .ok();
                }
                ClientMessage::DeleteCharacter(message) => {
                    let response = character_list
                        .get_mut(message.slot as usize)
                        .filter(|character| character.info.name == message.name)
                        .map_or_else(
                            || Err(DeleteCharacterError::Failed(message.name.clone())),
                            |character| {
                                if message.is_delete {
                                    if character.delete_time.is_none() {
                                        character.delete_time = Some(CharacterDeleteTime::new());
                                    }
                                } else {
                                    character.delete_time = None;
                                }

                                match character.save() {
                                    Ok(_) => log::info!("Saved character {}", character.info.name),
                                    Err(error) => log::error!(
                                        "Failed to save character {} with error {:?}",
                                        character.info.name,
                                        error
                                    ),
                                }

                                Ok(DeleteCharacterResponse {
                                    name: message.name.clone(),
                                    delete_time: character.delete_time.clone(),
                                })
                            },
                        );
                    world_client
                        .server_message_tx
                        .send(ServerMessage::DeleteCharacter(response))
                        .ok();
                }
                ClientMessage::SelectCharacter(message) => {
                    let response = character_list
                        .get_mut(message.slot as usize)
                        .filter(|character| character.info.name == message.name)
                        .map_or(Err(SelectCharacterError::Failed), |selected_character| {
                            // Set the selected_character for the login token
                            if let Some(token) = login_tokens
                                .tokens
                                .iter_mut()
                                .find(|t| t.token == world_client.login_token)
                            {
                                token.selected_character = selected_character.info.name.clone()
                            }

                            // Find the selected game server details
                            if let Some(selected_game_server) = world_client.selected_game_server {
                                if let Ok(server_info) = server_info_query.get(selected_game_server)
                                {
                                    Ok(JoinServerResponse {
                                        login_token: world_client.login_token,
                                        packet_codec_seed: server_info.packet_codec_seed,
                                        ip: server_info.ip.clone(),
                                        port: server_info.port,
                                    })
                                } else {
                                    Err(SelectCharacterError::Failed)
                                }
                            } else {
                                Err(SelectCharacterError::Failed)
                            }
                        });
                    world_client
                        .server_message_tx
                        .send(ServerMessage::SelectCharacter(response))
                        .ok();
                }
                _ => warn!("Received unimplemented client message {:?}", message),
            }
        }
    });
}
