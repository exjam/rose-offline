use legion::systems::CommandBuffer;
use legion::*;

use crate::game::components::{GameClient, LoginClient, ServerInfo, WorldClient};
use crate::game::messages::control::{ClientType, ControlMessage};
use crate::game::resources::{ControlChannel, GameServer, ServerList, WorldServer};

#[system]
pub fn control_server(
    cmd: &mut CommandBuffer,
    #[resource] channel: &mut ControlChannel,
    #[resource] server_list: &mut ServerList,
) {
    loop {
        match channel.control_rx.try_recv() {
            Ok(message) => match message {
                ControlMessage::AddClient {
                    client_type,
                    client_message_rx,
                    server_message_tx,
                    response_tx,
                } => {
                    let entity = match client_type {
                        ClientType::Login => {
                            cmd.push((LoginClient::new(client_message_rx, server_message_tx),))
                        }
                        ClientType::World => {
                            cmd.push((WorldClient::new(client_message_rx, server_message_tx),))
                        }
                        ClientType::Game => {
                            cmd.push((GameClient::new(client_message_rx, server_message_tx),))
                        }
                    };
                    response_tx.send(entity).unwrap();
                }
                ControlMessage::RemoveClient { entity } => {
                    cmd.remove(entity);
                }
                ControlMessage::AddWorldServer {
                    name,
                    ip,
                    port,
                    packet_codec_seed,
                    response_tx,
                } => {
                    let entity = cmd.push((ServerInfo {
                        name: name.clone(),
                        ip: ip.clone(),
                        port: port,
                        packet_codec_seed: packet_codec_seed,
                    },));
                    server_list.world_servers.push(WorldServer {
                        entity: entity,
                        name,
                        ip,
                        port,
                        packet_codec_seed,
                        channels: Vec::new(),
                    });
                    response_tx.send(entity).unwrap();
                }
                ControlMessage::AddGameServer {
                    world_server,
                    name,
                    ip,
                    port,
                    packet_codec_seed,
                    response_tx,
                } => {
                    let entity = cmd.push((ServerInfo {
                        name: name.clone(),
                        ip: ip.clone(),
                        port: port,
                        packet_codec_seed: packet_codec_seed,
                    },));
                    let world_server = server_list
                        .world_servers
                        .iter_mut()
                        .find(|s| s.entity == world_server)
                        .unwrap();
                    world_server.channels.push(GameServer {
                        entity: entity,
                        name,
                        ip,
                        port,
                        packet_codec_seed,
                    });
                    response_tx.send(entity).unwrap();
                }
                ControlMessage::RemoveServer { entity } => {
                    cmd.remove(entity);
                }
            },
            Err(_) => break,
        }
    }
}
