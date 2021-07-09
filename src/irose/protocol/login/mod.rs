use async_trait::async_trait;
use num_traits::FromPrimitive;
use std::convert::TryFrom;
use tokio::sync::oneshot;

use crate::game::messages::client::*;
use crate::game::messages::server::ServerMessage;
use crate::protocol::{Client, Packet, ProtocolClient, ProtocolError};

mod client_packets;
mod server_packets;
use client_packets::*;
use server_packets::*;

pub struct LoginClient {}

impl LoginClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &self,
        client: &mut Client<'_>,
        packet: Packet,
    ) -> Result<(), ProtocolError> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::Connect) => {
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest(ConnectionRequest {
                        login_token: 0u32,
                        password_md5: String::new(),
                        response_tx,
                    }))?;
                let packet = match response_rx.await? {
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
            Some(ClientPackets::LoginRequest) => {
                let login_request = PacketClientLoginRequest::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::LoginRequest(LoginRequest {
                        username: String::from(login_request.username),
                        password_md5: String::from(login_request.password_md5),
                        response_tx,
                    }))?;
                let packet = match response_rx.await? {
                    Ok(_) => {
                        let (response_tx, response_rx) = oneshot::channel();
                        client
                            .client_message_tx
                            .send(ClientMessage::GetWorldServerList(GetWorldServerList {
                                response_tx,
                            }))?;
                        let servers = response_rx.await?;
                        Packet::from(&PacketServerLoginReply {
                            result: LoginResult::Ok,
                            rights: 0x800,
                            pay_type: 0xff,
                            servers: &servers,
                        })
                    }
                    Err(LoginError::Failed) => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::Failed),
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
            Some(ClientPackets::ChannelList) => {
                let server_id = PacketClientChannelList::try_from(&packet)?.server_id;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::GetChannelList(GetChannelList {
                        server_id,
                        response_tx,
                    }))?;
                let packet = match response_rx.await? {
                    Ok(channels) => {
                        let mut channel_list: Vec<PacketServerChannelListItem> = Vec::new();
                        for (id, name) in &channels {
                            channel_list.push(PacketServerChannelListItem {
                                id: *id,
                                low_age: 0u8,
                                high_age: 100u8,
                                percent_full: 50u16,
                                name: &name,
                            });
                        }

                        Packet::from(&PacketServerChannelList {
                            server_id,
                            channels: &channel_list,
                        })
                    }
                    Err(_) => Packet::from(&PacketServerChannelList {
                        server_id,
                        channels: &[],
                    }),
                };
                client.connection.write_packet(packet).await?;
            }
            Some(ClientPackets::SelectServer) => {
                let select_server = PacketClientSelectServer::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::JoinServer(JoinServer {
                        server_id: select_server.server_id,
                        channel_id: select_server.channel_id,
                        response_tx,
                    }))?;

                let packet = match response_rx.await? {
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
            _ => println!("[LS] Unhandled packet {:#03X}", packet.command),
        }

        Ok(())
    }

    async fn handle_server_message(
        &self,
        _client: &mut Client<'_>,
        _message: ServerMessage,
    ) -> Result<(), ProtocolError> {
        panic!("Unimplemented message for irose login server!")
    }
}

#[async_trait]
impl ProtocolClient for LoginClient {
    async fn run_client(&self, client: &mut Client) -> Result<(), ProtocolError> {
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
                        return Err(ProtocolError::ServerInitiatedDisconnect);
                    }
                }
            };
        }
    }
}
