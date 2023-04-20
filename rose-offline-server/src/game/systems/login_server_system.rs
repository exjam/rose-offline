use bevy::ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use log::warn;

use crate::game::{
    components::{Account, LoginClient},
    messages::client::ClientMessage,
    messages::server::{ChannelListError, JoinServerError, LoginError, ServerMessage},
    resources::{LoginTokens, ServerList},
    storage::account::{AccountStorage, AccountStorageError},
};

pub fn login_server_authentication_system(
    mut commands: Commands,
    query: Query<(Entity, &LoginClient), Without<Account>>,
    login_tokens: Res<LoginTokens>,
    server_list: Res<ServerList>,
) {
    query.for_each(|(entity, login_client)| {
        if let Ok(message) = login_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::ConnectionRequest { .. } => {
                    login_client
                        .server_message_tx
                        .send(ServerMessage::ConnectionRequestSuccess {
                            packet_sequence_id: 123,
                        })
                        .ok();
                }
                ClientMessage::LoginRequest { username, password } => {
                    let login_result = if login_tokens.find_username_token(&username).is_some() {
                        Err(LoginError::AlreadyLoggedIn)
                    } else {
                        match AccountStorage::try_load(&username, &password) {
                            Ok(account) => Ok(account),
                            Err(error) => match error.downcast_ref::<AccountStorageError>() {
                                Some(AccountStorageError::NotFound) => {
                                    match AccountStorage::create(&username, &password) {
                                        Ok(account) => {
                                            log::info!("Created account {}", &username);
                                            Ok(account)
                                        }
                                        Err(error) => {
                                            log::info!(
                                                "Failed to create account {} with error {:?}",
                                                &username,
                                                error
                                            );
                                            Err(LoginError::InvalidAccount)
                                        }
                                    }
                                }
                                Some(AccountStorageError::InvalidPassword) => {
                                    Err(LoginError::InvalidPassword)
                                }
                                _ => {
                                    log::error!(
                                        "Failed to load account {} with error {:?}",
                                        &username,
                                        error
                                    );
                                    Err(LoginError::Failed)
                                }
                            },
                        }
                    };

                    let response = match login_result {
                        Ok(account) => {
                            commands.entity(entity).insert(Account::from(account));

                            ServerMessage::LoginSuccess {
                                server_list: server_list
                                    .world_servers
                                    .iter()
                                    .enumerate()
                                    .map(|(id, server)| (id as u32, server.name.clone()))
                                    .collect(),
                            }
                        }
                        Err(error) => ServerMessage::LoginError { error },
                    };

                    login_client.server_message_tx.send(response).ok();
                }
                _ => panic!("Received unexpected client message {:?}", message),
            }
        }
    });
}

pub fn login_server_system(
    mut query: Query<(Entity, &Account, &mut LoginClient)>,
    mut login_tokens: ResMut<LoginTokens>,
    server_list: Res<ServerList>,
) {
    query.for_each_mut(|(entity, account, mut login_client)| {
        if let Ok(message) = login_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GetChannelList { server_id } => {
                    let response = server_list.world_servers.get(server_id).map_or(
                        ServerMessage::ChannelListError {
                            error: ChannelListError::InvalidServerId { server_id },
                        },
                        |world_server| {
                            let mut channels = Vec::new();
                            for (id, channel) in world_server.channels.iter().enumerate() {
                                channels.push((id as u8, channel.name.clone()));
                            }
                            ServerMessage::ChannelList {
                                server_id,
                                channels,
                            }
                        },
                    );
                    login_client.server_message_tx.send(response).ok();
                }
                ClientMessage::JoinServer {
                    server_id,
                    channel_id,
                } => {
                    let response = server_list.world_servers.get(server_id).map_or(
                        ServerMessage::JoinServerError {
                            error: JoinServerError::InvalidServerId,
                        },
                        |world_server| {
                            world_server.channels.get(channel_id).map_or(
                                ServerMessage::JoinServerError {
                                    error: JoinServerError::InvalidChannelId,
                                },
                                |game_server| {
                                    login_client.login_token = login_tokens.generate(
                                        account.name.clone(),
                                        entity,
                                        world_server.entity,
                                        game_server.entity,
                                    );
                                    ServerMessage::JoinServerSuccess {
                                        login_token: login_client.login_token,
                                        packet_codec_seed: world_server.packet_codec_seed,
                                        ip: world_server.ip.clone(),
                                        port: world_server.port,
                                    }
                                },
                            )
                        },
                    );

                    login_client.server_message_tx.send(response).ok();
                }
                _ => warn!("[LS] Received unimplemented client message {:?}", message),
            }
        }
    });
}
