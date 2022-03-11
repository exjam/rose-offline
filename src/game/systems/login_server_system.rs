use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use log::warn;

use crate::game::{
    components::{Account, LoginClient},
    messages::client::ClientMessage,
    messages::{
        client::GetChannelList,
        server::{
            ChannelList, ChannelListError, ConnectionResponse, JoinServerError, JoinServerResponse,
            LoginError, LoginResponse, ServerMessage,
        },
    },
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
                ClientMessage::ConnectionRequest(_) => {
                    login_client
                        .server_message_tx
                        .send(ServerMessage::ConnectionResponse(Ok(ConnectionResponse {
                            packet_sequence_id: 123,
                        })))
                        .ok();
                }
                ClientMessage::LoginRequest(message) => {
                    let response = if login_tokens
                        .find_username_token(&message.username)
                        .is_some()
                    {
                        Err(LoginError::AlreadyLoggedIn)
                    } else {
                        match AccountStorage::try_load(&message.username, &message.password_md5) {
                            Ok(account) => {
                                commands.entity(entity).insert(Account::from(account));
                                let mut servers = Vec::new();
                                for (id, server) in server_list.world_servers.iter().enumerate() {
                                    servers.push((id as u32, server.name.clone()));
                                }
                                Ok(LoginResponse {
                                    server_list: servers,
                                })
                            }
                            Err(error) => Err(match error {
                                AccountStorageError::NotFound => LoginError::InvalidAccount,
                                AccountStorageError::InvalidPassword => LoginError::InvalidPassword,
                                _ => LoginError::Failed,
                            }),
                        }
                    };
                    login_client
                        .server_message_tx
                        .send(ServerMessage::LoginResponse(response))
                        .ok();
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
                ClientMessage::GetChannelList(GetChannelList { server_id }) => {
                    let response = server_list
                        .world_servers
                        .get(server_id)
                        .ok_or(ChannelListError::InvalidServerId(server_id))
                        .map(|world_server| {
                            let mut channels = Vec::new();
                            for (id, channel) in world_server.channels.iter().enumerate() {
                                channels.push((id as u8, channel.name.clone()));
                            }
                            ChannelList {
                                server_id,
                                channels,
                            }
                        });
                    login_client
                        .server_message_tx
                        .send(ServerMessage::ChannelList(response))
                        .ok();
                }
                ClientMessage::JoinServer(message) => {
                    let response = server_list
                        .world_servers
                        .get(message.server_id as usize)
                        .ok_or(JoinServerError::InvalidServerId)
                        .and_then(|world_server| {
                            world_server
                                .channels
                                .get(message.channel_id as usize)
                                .ok_or(JoinServerError::InvalidChannelId)
                                .map(|game_server| {
                                    login_client.login_token = login_tokens.generate(
                                        account.name.clone(),
                                        entity,
                                        world_server.entity,
                                        game_server.entity,
                                    );
                                    JoinServerResponse {
                                        login_token: login_client.login_token,
                                        packet_codec_seed: world_server.packet_codec_seed,
                                        ip: world_server.ip.clone(),
                                        port: world_server.port,
                                    }
                                })
                        });

                    login_client
                        .server_message_tx
                        .send(ServerMessage::JoinServer(response))
                        .ok();
                }
                _ => warn!("Received unimplemented client message {:?}", message),
            }
        }
    });
}
