use bevy::ecs::prelude::{Commands, EventWriter, Res, ResMut};

use crate::game::{
    components::{GameClient, LoginClient, ServerInfo, WorldClient},
    events::SaveEvent,
    messages::control::{ClientType, ControlMessage},
    resources::{ControlChannel, GameServer, LoginTokens, ServerList, WorldServer},
};

pub fn control_server_system(
    mut commands: Commands,
    channel: Res<ControlChannel>,
    mut login_tokens: ResMut<LoginTokens>,
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
                        .spawn(LoginClient::new(client_message_rx, server_message_tx))
                        .id(),
                    ClientType::World => commands
                        .spawn(WorldClient::new(client_message_rx, server_message_tx))
                        .id(),
                    ClientType::Game => commands
                        .spawn(GameClient::new(client_message_rx, server_message_tx))
                        .id(),
                };
                response_tx.send(entity).unwrap();
            }
            ControlMessage::RemoveClient {
                client_type,
                entity,
            } => match client_type {
                ClientType::Login => {
                    for login_token in login_tokens.tokens.iter_mut() {
                        if login_token.login_client == Some(entity) {
                            login_token.login_client = None;
                        }
                    }

                    commands.entity(entity).despawn();
                }
                ClientType::World => {
                    for (index, login_token) in login_tokens.tokens.iter_mut().enumerate() {
                        if login_token.world_client == Some(entity) {
                            login_token.world_client = None;
                        }

                        if login_token.game_client.is_none() && login_token.world_client.is_none() {
                            login_tokens.tokens.remove(index);
                            break;
                        }
                    }

                    commands.entity(entity).despawn();
                }
                ClientType::Game => {
                    for (index, login_token) in login_tokens.tokens.iter_mut().enumerate() {
                        if login_token.game_client == Some(entity) {
                            login_token.game_client = None;
                        }

                        if login_token.game_client.is_none() && login_token.world_client.is_none() {
                            login_tokens.tokens.remove(index);
                            break;
                        }
                    }

                    // Let the save system handle despawning the entity
                    save_events.send(SaveEvent::with_character(entity, true));
                    commands.entity(entity).remove::<GameClient>();
                }
            },
            ControlMessage::AddWorldServer {
                name,
                ip,
                port,
                packet_codec_seed,
                response_tx,
            } => {
                let entity = commands
                    .spawn(ServerInfo {
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
                    .spawn(ServerInfo {
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
