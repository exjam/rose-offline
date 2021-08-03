use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut, Without};
use log::warn;

use crate::{
    data::account::{AccountStorage, AccountStorageError},
    game::{
        components::{Account, LoginClient},
        messages::client::{
            ClientMessage, ConnectionRequestResponse, GetChannelListError, JoinServerError,
            JoinServerResponse, LoginError,
        },
        resources::{LoginTokens, ServerList},
    },
};

pub fn login_server_authentication_system(
    mut commands: Commands,
    query: Query<(Entity, &LoginClient), Without<Account>>,
) {
    query.for_each(|(entity, login_client)| {
        if let Ok(message) = login_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::ConnectionRequest(message) => {
                    message
                        .response_tx
                        .send(Ok(ConnectionRequestResponse {
                            packet_sequence_id: 123,
                        }))
                        .ok();
                }
                ClientMessage::LoginRequest(message) => {
                    let result =
                        match AccountStorage::try_load(&message.username, &message.password_md5) {
                            Ok(account) => {
                                commands.entity(entity).insert(Account::from(account));
                                Ok(())
                            }
                            Err(error) => Err(match error {
                                AccountStorageError::NotFound => LoginError::InvalidAccount,
                                AccountStorageError::InvalidPassword => LoginError::InvalidPassword,
                                _ => LoginError::Failed,
                            }),
                        };
                    message.response_tx.send(result).ok();
                }
                _ => panic!("Received unexpected client message {:?}", message),
            }
        }
    });
}

pub fn login_server_system(
    mut query: Query<(&Account, &mut LoginClient)>,
    mut login_tokens: ResMut<LoginTokens>,
    server_list: Res<ServerList>,
) {
    query.for_each_mut(|(account, mut login_client)| {
        if let Ok(message) = login_client.client_message_rx.try_recv() {
            match message {
                ClientMessage::GetWorldServerList(message) => {
                    let mut servers = Vec::new();
                    for (id, server) in server_list.world_servers.iter().enumerate() {
                        servers.push((id as u32, server.name.clone()));
                    }
                    message.response_tx.send(servers).ok();
                }
                ClientMessage::GetChannelList(message) => {
                    let response = server_list
                        .world_servers
                        .get(message.server_id as usize)
                        .ok_or(GetChannelListError::InvalidServerId)
                        .map(|world_server| {
                            let mut channels = Vec::new();
                            for (id, channel) in world_server.channels.iter().enumerate() {
                                channels.push((id as u8, channel.name.clone()));
                            }
                            channels
                        });
                    message.response_tx.send(response).ok();
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

                    message.response_tx.send(response).ok();
                }
                _ => warn!("Received unimplemented client message {:?}", message),
            }
        }
    });
}
