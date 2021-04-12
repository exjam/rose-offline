use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use crate::game::data::account::{AccountStorage, AccountStorageError};
use crate::game::messages::client::{
    ClientMessage, ConnectionRequestResponse, GetChannelListError, JoinServerError,
    JoinServerResponse, LoginError,
};
use crate::game::{
    components::{Account, LoginClient},
    resources::LoginTokens,
    resources::ServerList,
};

#[system(for_each)]
pub fn login_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut LoginClient,
    #[resource] login_tokens: &mut LoginTokens,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
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
                            cmd.add_component(*entity, Account::from(account));
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
            _ => {
                client.pending_messages.push_back(message);
            }
        }
    }
}

#[system(for_each)]
pub fn login_server(
    account: &Account,
    client: &mut LoginClient,
    #[resource] server_list: &ServerList,
    #[resource] login_tokens: &mut LoginTokens,
) {
    while let Some(message) = client.pending_messages.pop_front() {
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
                                client.login_token = login_tokens.generate(
                                    account.name.clone(),
                                    world_server.entity,
                                    game_server.entity,
                                );
                                JoinServerResponse {
                                    login_token: client.login_token,
                                    packet_codec_seed: world_server.packet_codec_seed,
                                    ip: world_server.ip.clone(),
                                    port: world_server.port,
                                }
                            })
                    });

                message.response_tx.send(response).ok();
            }
            _ => {
                panic!("Unhandled client message for login server!");
            }
        }
    }
}
