use bevy_ecs::prelude::{Commands, EventWriter, Res, ResMut};

use crate::game::{
    components::{GameClient, LoginClient, ServerInfo, WorldClient},
    events::SaveEvent,
    messages::control::{ClientType, ControlMessage},
    resources::{ControlChannel, GameServer, ServerList, WorldServer},
};

pub fn control_server_system(
    mut commands: Commands,
    channel: Res<ControlChannel>,
    mut server_list: ResMut<ServerList>,
    mut save_events: EventWriter<SaveEvent>,
) {
    while let Ok(message) = channel.control_rx.try_recv() {
        match message {
            ControlMessage::AddClient {
                client_type,
                client_message_rx,
                server_message_tx,
                response_tx,
            } => {
                let entity = match client_type {
                    ClientType::Login => commands
                        .spawn()
                        .insert(LoginClient::new(client_message_rx, server_message_tx))
                        .id(),
                    ClientType::World => commands
                        .spawn()
                        .insert(WorldClient::new(client_message_rx, server_message_tx))
                        .id(),
                    ClientType::Game => commands
                        .spawn()
                        .insert(GameClient::new(client_message_rx, server_message_tx))
                        .id(),
                };
                response_tx.send(entity).unwrap();
            }
            ControlMessage::RemoveClient {
                client_type,
                entity,
            } => match client_type {
                ClientType::Game => save_events.send(SaveEvent::with_character(entity, true)),
                _ => commands.entity(entity).despawn(),
            },
            ControlMessage::AddWorldServer {
                name,
                ip,
                port,
                packet_codec_seed,
                response_tx,
            } => {
                let entity = commands
                    .spawn()
                    .insert(ServerInfo {
                        name: name.clone(),
                        ip: ip.clone(),
                        port,
                        packet_codec_seed,
                    })
                    .id();
                server_list.world_servers.push(WorldServer {
                    entity,
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
                let entity = commands
                    .spawn()
                    .insert(ServerInfo {
                        name: name.clone(),
                        ip: ip.clone(),
                        port,
                        packet_codec_seed,
                    })
                    .id();
                let world_server = server_list
                    .world_servers
                    .iter_mut()
                    .find(|s| s.entity == world_server)
                    .unwrap();
                world_server.channels.push(GameServer {
                    entity,
                    name,
                    ip,
                    port,
                    packet_codec_seed,
                });
                response_tx.send(entity).unwrap();
            }
            ControlMessage::RemoveServer { entity } => {
                commands.entity(entity).despawn();
            }
        }
    }
}
