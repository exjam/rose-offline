use async_trait::async_trait;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::messages::{
    client::{ClientMessage, ConnectionRequest, GetChannelList, JoinServer, LoginRequest},
    server::{
        ChannelList, ChannelListError, JoinServerError, LoginError, LoginResponse, ServerMessage,
    },
};
use rose_network_common::{Packet, PacketError};
use rose_network_irose::{login_client_packets::*, login_server_packets::*};

use crate::protocol::{Client, ProtocolServer, ProtocolServerError};

pub struct LoginServer;

impl LoginServer {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &mut self,
        client: &mut Client<'_>,
        packet: Packet,
    ) -> Result<(), anyhow::Error> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::Connect) => {
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest(ConnectionRequest {
                        login_token: 0u32,
                        password_md5: String::new(),
                    }))?;
            }
            Some(ClientPackets::LoginRequest) => {
                let login_request = PacketClientLoginRequest::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::LoginRequest(LoginRequest {
                        username: String::from(login_request.username),
                        password_md5: String::from(login_request.password_md5),
                    }))?;
            }
            Some(ClientPackets::ChannelList) => {
                let server_id = PacketClientChannelList::try_from(&packet)?.server_id;
                client
                    .client_message_tx
                    .send(ClientMessage::GetChannelList(GetChannelList { server_id }))?;
            }
            Some(ClientPackets::SelectServer) => {
                let select_server = PacketClientSelectServer::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::JoinServer(JoinServer {
                        server_id: select_server.server_id,
                        channel_id: select_server.channel_id,
                    }))?;
            }
            _ => return Err(PacketError::InvalidPacket.into()),
        }

        Ok(())
    }

    async fn handle_server_message(
        &mut self,
        client: &mut Client<'_>,
        message: ServerMessage,
    ) -> Result<(), anyhow::Error> {
        match message {
            ServerMessage::ConnectionResponse(response) => {
                let packet = match response {
                    Ok(result) => Packet::from(&PacketConnectionReply {
                        status: ConnectionResult::Accepted,
                        packet_sequence_id: result.packet_sequence_id,
                    }),
                    Err(_) => Packet::from(&PacketConnectionReply {
                        status: ConnectionResult::Disconnect,
                        packet_sequence_id: 0u32,
                    }),
                };
                client.connection.write_packet(packet).await?;
            }
            ServerMessage::LoginResponse(response) => {
                let packet = match response {
                    Ok(LoginResponse { server_list }) => Packet::from(&PacketServerLoginReply {
                        result: LoginResult::Ok,
                        rights: 0x800,
                        pay_type: 0xff,
                        servers: server_list,
                    }),
                    Err(LoginError::Failed) => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::Failed),
                    ),
                    Err(LoginError::AlreadyLoggedIn) => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::AlreadyLoggedIn),
                    ),
                    Err(LoginError::InvalidAccount) => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::UnknownAccount),
                    ),
                    Err(LoginError::InvalidPassword) => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::InvalidPassword),
                    ),
                };
                client.connection.write_packet(packet).await?;
            }
            ServerMessage::ChannelList(message) => {
                let packet = match message {
                    Ok(ChannelList {
                        server_id,
                        channels,
                    }) => {
                        let mut channel_list: Vec<PacketServerChannelListItem> = Vec::new();
                        for (id, name) in &channels {
                            channel_list.push(PacketServerChannelListItem {
                                id: *id,
                                low_age: 0u8,
                                high_age: 100u8,
                                percent_full: 50u16,
                                name,
                            });
                        }

                        Packet::from(&PacketServerChannelList {
                            server_id,
                            channels: channel_list,
                        })
                    }
                    Err(ChannelListError::InvalidServerId(server_id)) => {
                        Packet::from(&PacketServerChannelList {
                            server_id,
                            channels: Vec::new(),
                        })
                    }
                };
                client.connection.write_packet(packet).await?;
            }
            ServerMessage::JoinServer(message) => {
                let packet = match message {
                    Ok(response) => Packet::from(&PacketServerSelectServer {
                        result: SelectServerResult::Ok,
                        login_token: response.login_token,
                        packet_codec_seed: response.packet_codec_seed,
                        ip: &response.ip,
                        port: response.port,
                    }),
                    Err(JoinServerError::InvalidChannelId) => Packet::from(
                        &PacketServerSelectServer::with_result(SelectServerResult::InvalidChannel),
                    ),
                    Err(JoinServerError::InvalidServerId) => Packet::from(
                        &PacketServerSelectServer::with_result(SelectServerResult::Failed),
                    ),
                };
                client.connection.write_packet(packet).await?;
            }
            _ => panic!("Received unexpected server message for login server"),
        }

        Ok(())
    }
}

#[async_trait]
impl ProtocolServer for LoginServer {
    async fn run_client(&mut self, client: &mut Client) -> Result<(), anyhow::Error> {
        loop {
            tokio::select! {
                packet = client.connection.read_packet() => {
                    match packet {
                        Ok(packet) => {
                            self.handle_packet(client, packet).await?;
                        },
                        Err(error) => {
                            return Err(error);
                        }
                    }
                },
                server_message = client.server_message_rx.recv() => {
                    if let Some(message) = server_message {
                        self.handle_server_message(client, message).await?;
                    } else {
                        return Err(ProtocolServerError::ServerInitiatedDisconnect.into());
                    }
                }
            };
        }
    }
}
