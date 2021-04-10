use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use crate::game::messages::client::{
    ClientMessage, ConnectionRequestResponse, GetChannelListError, JoinServerError, LoginError,
};
use crate::game::{
    components::{Account, AccountError, LoginClient},
    resources::ServerList,
};

#[system]
pub fn login_server(
    world: &SubWorld,
    cmd: &mut CommandBuffer,
    clients: &mut Query<(Entity, &LoginClient)>,
    #[resource] server_list: &ServerList,
) {
    for (entity, client) in clients.iter(world) {
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
                    let result = match Account::try_load(&message.username, &message.password_md5) {
                        Ok(account) => {
                            // Add Account component to this client entity
                            cmd.add_component(*entity, account);
                            Ok(())
                        }
                        Err(error) => Err(match error {
                            AccountError::NotFound => LoginError::InvalidAccount,
                            AccountError::InvalidPassword => LoginError::InvalidPassword,
                            _ => LoginError::Failed,
                        }),
                    };
                    message.response_tx.send(result).ok();
                }
                ClientMessage::GetWorldServerList(message) => {
                    let mut servers = Vec::new();
                    for (id, server) in server_list.world_servers.iter().enumerate() {
                        servers.push((id as u32, server.name.clone()));
                    }
                    message.response_tx.send(servers).ok();
                }
                ClientMessage::GetChannelList(message) => {
                    if let Some(world_server) =
                        server_list.world_servers.get(message.server_id as usize)
                    {
                        let mut channels = Vec::new();
                        for (id, channel) in world_server.channels.iter().enumerate() {
                            channels.push((id as u8, channel.name.clone()));
                        }
                        message.response_tx.send(Ok(channels)).ok();
                    } else {
                        message
                            .response_tx
                            .send(Err(GetChannelListError::InvalidServerId))
                            .ok();
                    }
                }
                ClientMessage::JoinServer(message) => {
                    if let Some(world_server) =
                        server_list.world_servers.get(message.server_id as usize)
                    {
                        if let Some(game_server) =
                            world_server.channels.get(message.channel_id as usize)
                        {
                            // TODO:
                        } else {
                            message
                                .response_tx
                                .send(Err(JoinServerError::InvalidChannelId))
                                .ok();
                        }
                    } else {
                        message
                            .response_tx
                            .send(Err(JoinServerError::InvalidServerId))
                            .ok();
                    }
                }
            }
        }
    }
}
