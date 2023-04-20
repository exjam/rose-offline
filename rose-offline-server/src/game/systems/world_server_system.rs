use bevy::{
    ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without},
    prelude::EventWriter,
};
use log::warn;

use rose_game_common::data::Password;

use crate::game::{
    components::{Account, CharacterDeleteTime, CharacterList, ServerInfo, WorldClient},
    events::ClanEvent,
    messages::{
        client::ClientMessage,
        server::{CharacterListItem, ConnectionRequestError, CreateCharacterError, ServerMessage},
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
) -> Result<u32, ConnectionRequestError> {
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

    Ok(123)
}

pub fn world_server_authentication_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorldClient), Without<Account>>,
    mut login_tokens: ResMut<LoginTokens>,
) {
    query.for_each_mut(|(entity, mut world_client)| {
        if let Ok(message) = world_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::ConnectionRequest {
                    login_token,
                    password,
                } => {
                    let response = match handle_world_connection_request(
                        &mut commands,
                        login_tokens.as_mut(),
                        entity,
                        world_client.as_mut(),
                        login_token,
                        &password,
                    ) {
                        Ok(packet_sequence_id) => {
                            ServerMessage::ConnectionRequestSuccess { packet_sequence_id }
                        }
                        Err(error) => ServerMessage::ConnectionRequestError { error },
                    };
                    world_client.server_message_tx.send(response).ok();
                }
                _ => panic!("Received unexpected client message {:?}", message),
            }
        }
    });
}

pub fn world_server_system(
    mut world_client_query: Query<(&mut WorldClient, &mut Account, &mut CharacterList)>,
    server_info_query: Query<&ServerInfo>,
    mut login_tokens: ResMut<LoginTokens>,
    game_data: Res<GameData>,
    mut clan_events: EventWriter<ClanEvent>,
) {
    world_client_query.for_each_mut(|(world_client, mut account, mut character_list)| {
        if let Ok(message) = world_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GetCharacterList => {
                    world_client
                        .server_message_tx
                        .send(ServerMessage::CharacterList {
                            character_list: character_list
                                .iter()
                                .map(|character| CharacterListItem {
                                    info: character.info.clone(),
                                    level: character.level,
                                    delete_time: character.delete_time,
                                    equipment: character.equipment.clone(),
                                })
                                .collect(),
                        })
                        .ok();
                }
                ClientMessage::CreateCharacter {
                    gender,
                    hair,
                    face,
                    name,
                    birth_stone,
                    ..
                } => {
                    let response = if account.character_names.len() >= 5 {
                        ServerMessage::CreateCharacterError {
                            error: CreateCharacterError::NoMoreSlots,
                        }
                    } else if name.len() < 4 || name.len() > 20 {
                        ServerMessage::CreateCharacterError {
                            error: CreateCharacterError::InvalidValue,
                        }
                    } else if CharacterStorage::exists(&name) {
                        ServerMessage::CreateCharacterError {
                            error: CreateCharacterError::AlreadyExists,
                        }
                    } else {
                        match game_data.character_creator.create(
                            name.clone(),
                            gender,
                            birth_stone as u8,
                            face as u8,
                            hair as u8,
                        ) {
                            Ok(character) => {
                                if let Err(error) = character.try_create() {
                                    log::error!(
                                        "Failed to create character {} with error {:?}",
                                        &name,
                                        error
                                    );
                                    ServerMessage::CreateCharacterError {
                                        error: CreateCharacterError::Failed,
                                    }
                                } else {
                                    let character_slot = account.character_names.len();
                                    account.character_names.push(character.info.name.clone());
                                    AccountStorage::from(&*account).save().ok();
                                    character_list.push(character);
                                    ServerMessage::CreateCharacterSuccess { character_slot }
                                }
                            }
                            Err(error) => {
                                log::error!(
                                    "Failed to create character {} with error {:?}",
                                    &name,
                                    error
                                );
                                ServerMessage::CreateCharacterError {
                                    error: CreateCharacterError::InvalidValue,
                                }
                            }
                        }
                    };

                    world_client.server_message_tx.send(response).ok();
                }
                ClientMessage::DeleteCharacter {
                    slot,
                    name,
                    is_delete,
                } => {
                    let response = character_list
                        .get_mut(slot as usize)
                        .filter(|character| character.info.name == name)
                        .map_or_else(
                            || ServerMessage::DeleteCharacterError { name: name.clone() },
                            |character| {
                                if is_delete {
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

                                if let Some(delete_time) = character.delete_time {
                                    ServerMessage::DeleteCharacterStart {
                                        name: name.clone(),
                                        delete_time,
                                    }
                                } else {
                                    ServerMessage::DeleteCharacterCancel { name: name.clone() }
                                }
                            },
                        );
                    world_client.server_message_tx.send(response).ok();
                }
                ClientMessage::SelectCharacter { slot, name } => {
                    let response = character_list
                        .get_mut(slot as usize)
                        .filter(|character| character.info.name == name)
                        .map_or(ServerMessage::SelectCharacterError, |selected_character| {
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
                                    ServerMessage::SelectCharacterSuccess {
                                        login_token: world_client.login_token,
                                        packet_codec_seed: server_info.packet_codec_seed,
                                        ip: server_info.ip.clone(),
                                        port: server_info.port,
                                    }
                                } else {
                                    ServerMessage::SelectCharacterError
                                }
                            } else {
                                ServerMessage::SelectCharacterError
                            }
                        });
                    world_client.server_message_tx.send(response).ok();
                }
                ClientMessage::ClanGetMemberList => {
                    if let Some(game_client_entity) = world_client.game_client_entity {
                        clan_events.send(ClanEvent::GetMemberList {
                            entity: game_client_entity,
                        });
                    }
                }
                _ => warn!("[WS] Received unimplemented client message {:?}", message),
            }
        }
    });
}
