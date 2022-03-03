use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use log::warn;

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

fn handle_world_connection_request(
    commands: &mut Commands,
    login_tokens: &mut LoginTokens,
    entity: Entity,
    world_client: &mut WorldClient,
    token_id: u32,
    password_md5: &str,
) -> Result<ConnectionRequestResponse, ConnectionRequestError> {
    let login_token = login_tokens
        .get_token_mut(token_id)
        .ok_or(ConnectionRequestError::InvalidToken)?;
    if login_token.world_client.is_some() || login_token.game_client.is_some() {
        return Err(ConnectionRequestError::InvalidToken);
    }

    let mut account = AccountStorage::try_load(&login_token.username, password_md5).map_err(
        |error| match error {
            AccountStorageError::InvalidPassword => ConnectionRequestError::InvalidPassword,
            _ => ConnectionRequestError::Failed,
        },
    )?;

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

    Ok(ConnectionRequestResponse {
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
                    message
                        .response_tx
                        .send(handle_world_connection_request(
                            &mut commands,
                            login_tokens.as_mut(),
                            entity,
                            world_client.as_mut(),
                            message.login_token,
                            &message.password_md5,
                        ))
                        .ok();
                }
                _ => panic!("Received unexpected client message {:?}", message),
            }
        }
    });
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

pub fn world_server_system(
    mut world_client_query: Query<(&mut WorldClient, &mut Account, &mut CharacterList)>,
    server_info_query: Query<&ServerInfo>,
    mut login_tokens: ResMut<LoginTokens>,
    game_data: Res<GameData>,
) {
    world_client_query.for_each_mut(|(world_client, mut account, mut character_list)| {
        if let Ok(message) = world_client.client_message_rx.try_recv() {
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
                    message.response_tx.send(response).ok();
                }
                _ => warn!("Received unimplemented client message {:?}", message),
            }
        }
    });
}