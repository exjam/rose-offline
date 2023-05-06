use async_trait::async_trait;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::{
    data::Password,
    messages::{
        client::ClientMessage,
        server::{ChannelListError, JoinServerError, LoginError, ServerMessage},
    },
};
use rose_network_common::{Packet, PacketError};
use rose_network_irose::{login_client_packets::*, login_server_packets::*};

use crate::{
    implement_protocol_server,
    protocol::{Client, ProtocolServer, ProtocolServerError},
};

pub struct LoginServer;

impl LoginServer {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_packet(
        &mut self,
        client: &mut Client<'_>,
        packet: &Packet,
    ) -> Result<(), anyhow::Error> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::Connect) => {
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest {
                        login_token: 0u32,
                        password: Password::Plaintext(String::new()),
                    })?;
            }
            Some(ClientPackets::LoginRequest) => {
                let request = PacketClientLoginRequest::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::LoginRequest {
                    username: String::from(request.username),
                    password: Password::Md5(request.password_md5.into()),
                })?;
            }
            Some(ClientPackets::ChannelList) => {
                let server_id = PacketClientChannelList::try_from(packet)?.server_id;
                client
                    .client_message_tx
                    .send(ClientMessage::GetChannelList { server_id })?;
            }
            Some(ClientPackets::SelectServer) => {
                let select_server = PacketClientSelectServer::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::JoinServer {
                    server_id: select_server.server_id,
                    channel_id: select_server.channel_id,
                })?;
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
            ServerMessage::ConnectionRequestSuccess { packet_sequence_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketConnectionReply {
                        status: ConnectionResult::Accepted,
                        packet_sequence_id,
                    }))
                    .await?;
            }
            ServerMessage::ConnectionRequestError { .. } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketConnectionReply {
                        status: ConnectionResult::Disconnect,
                        packet_sequence_id: 0u32,
                    }))
                    .await?;
            }
            ServerMessage::LoginSuccess { server_list } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLoginReply {
                        result: LoginResult::Ok,
                        rights: 0x800,
                        pay_type: 0xff,
                        servers: server_list,
                    }))
                    .await?;
            }
            ServerMessage::LoginError { error } => {
                let packet = match error {
                    LoginError::Failed => Packet::from(&PacketServerLoginReply::with_error_result(
                        LoginResult::Failed,
                    )),
                    LoginError::AlreadyLoggedIn => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::AlreadyLoggedIn),
                    ),
                    LoginError::InvalidAccount => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::UnknownAccount),
                    ),
                    LoginError::InvalidPassword => Packet::from(
                        &PacketServerLoginReply::with_error_result(LoginResult::InvalidPassword),
                    ),
                };
                client.connection.write_packet(packet).await?;
            }
            ServerMessage::ChannelList {
                server_id,
                channels,
            } => {
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

                client
                    .connection
                    .write_packet(Packet::from(&PacketServerChannelList {
                        server_id,
                        channels: channel_list,
                    }))
                    .await?;
            }
            ServerMessage::ChannelListError { error } => {
                let ChannelListError::InvalidServerId { server_id } = error;
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerChannelList {
                        server_id,
                        channels: Vec::new(),
                    }))
                    .await?;
            }
            ServerMessage::JoinServerSuccess {
                login_token,
                packet_codec_seed,
                ref ip,
                port,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSelectServer {
                        result: SelectServerResult::Ok,
                        login_token,
                        packet_codec_seed,
                        ip,
                        port,
                    }))
                    .await?;
            }
            ServerMessage::JoinServerError { error } => {
                let packet = match error {
                    JoinServerError::InvalidServerId => Packet::from(
                        &PacketServerSelectServer::with_result(SelectServerResult::InvalidChannel),
                    ),
                    JoinServerError::InvalidChannelId => Packet::from(
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

implement_protocol_server! { LoginServer }
