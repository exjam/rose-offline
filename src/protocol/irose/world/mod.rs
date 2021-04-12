use std::convert::TryFrom;

use crate::game::messages::client::*;
use crate::game::messages::server::ServerMessage;
use crate::protocol::{packet::Packet, Client, ProtocolClient, ProtocolError};
use async_trait::async_trait;
use num_traits::FromPrimitive;

mod client_packets;
mod server_packets;

use client_packets::*;
use server_packets::*;
use tokio::sync::oneshot;

pub struct WorldClient {}

impl WorldClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet<'a>(
        &self,
        client: &mut Client<'a>,
        packet: Packet,
    ) -> Result<(), ProtocolError> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::ConnectRequest) => {
                let request = PacketClientConnectRequest::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest(ConnectionRequest {
                        login_token: request.login_token,
                        password_md5: String::from(request.password_md5),
                        response_tx: response_tx,
                    }))?;
                let packet = match response_rx.await? {
                    Ok(result) => Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Ok,
                        packet_sequence_id: result.packet_sequence_id,
                        pay_flags: 0xff,
                    }),
                    Err(_) => Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Failed,
                        packet_sequence_id: 0,
                        pay_flags: 0,
                    }),
                };
                client.connection.write_packet(packet).await?;
            }
            Some(ClientPackets::CharacterListRequest) => {
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::GetCharacterList(GetCharacterList {
                        response_tx: response_tx,
                    }))?;
                let response = response_rx.await?;
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCharacterList {
                        characters: &response[..],
                    }))
                    .await?;
            }
            Some(ClientPackets::CreateCharacter) => {
                let request = PacketClientCreateCharacter::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::CreateCharacter(CreateCharacter {
                        gender: request.gender,
                        birth_stone: request.birth_stone,
                        hair: request.hair,
                        face: request.face,
                        name: String::from(request.name),
                        response_tx: response_tx,
                    }))?;
                let response = match response_rx.await? {
                    Ok(slot) => Packet::from(&PacketServerCreateCharacterReply {
                        result: CreateCharacterResult::Ok,
                        is_platinum: slot >= 3,
                    }),
                    Err(CreateCharacterError::Failed) => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::Failed,
                            is_platinum: false,
                        })
                    }
                    Err(CreateCharacterError::AlreadyExists) => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::NameAlreadyExists,
                            is_platinum: false,
                        })
                    }
                    Err(CreateCharacterError::InvalidValue) => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::InvalidValue,
                            is_platinum: false,
                        })
                    }
                    Err(CreateCharacterError::NoMoreSlots) => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::NoMoreSlots,
                            is_platinum: false,
                        })
                    }
                };
                client.connection.write_packet(response).await?;
            }
            _ => return Err(ProtocolError::InvalidPacket),
        }

        Ok(())
    }

    async fn handle_server_message<'a>(
        &self,
        client: &mut Client<'a>,
        message: ServerMessage,
    ) -> Result<(), ProtocolError> {
        match message {
            _ => {
                panic!("Unimplemented message for irose world server!")
            }
        }
    }
}

#[async_trait]
impl ProtocolClient for WorldClient {
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
                Some(message) = client.server_message_rx.recv() => {
                    self.handle_server_message(client, message).await?;
                }
            };
        }
    }
}
