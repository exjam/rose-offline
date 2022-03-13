use async_trait::async_trait;
use log::warn;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::messages::{
    client::{ClientMessage, ConnectionRequest, CreateCharacter, DeleteCharacter, SelectCharacter},
    server::{
        CreateCharacterError, CreateCharacterResponse, DeleteCharacterError,
        DeleteCharacterResponse, ServerMessage,
    },
};
use rose_network_common::{Packet, PacketError};
use rose_network_irose::{world_client_packets::*, world_server_packets::*};

use crate::protocol::{Client, ProtocolServer, ProtocolServerError};

pub struct WorldServer;

impl WorldServer {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &mut self,
        client: &mut Client<'_>,
        packet: Packet,
    ) -> Result<(), anyhow::Error> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::ConnectRequest) => {
                let request = PacketClientConnectRequest::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest(ConnectionRequest {
                        login_token: request.login_token,
                        password_md5: String::from(request.password_md5),
                    }))?;
            }
            Some(ClientPackets::CharacterListRequest) => {
                client
                    .client_message_tx
                    .send(ClientMessage::GetCharacterList)?;
            }
            Some(ClientPackets::CreateCharacter) => {
                let request = PacketClientCreateCharacter::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CreateCharacter(CreateCharacter {
                        gender: request.gender,
                        birth_stone: request.birth_stone,
                        hair: request.hair,
                        face: request.face,
                        name: String::from(request.name),
                    }))?;
            }
            Some(ClientPackets::DeleteCharacter) => {
                let request = PacketClientDeleteCharacter::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::DeleteCharacter(DeleteCharacter {
                        slot: request.slot,
                        name: String::from(request.name),
                        is_delete: request.is_delete,
                    }))?;
            }
            Some(ClientPackets::SelectCharacter) => {
                let request = PacketClientSelectCharacter::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::SelectCharacter(SelectCharacter {
                        slot: request.slot,
                        name: String::from(request.name),
                    }))?;
            }
            _ => warn!(
                "[WS] Unhandled packet [{:#03X}] {:02x?}",
                packet.command,
                &packet.data[..]
            ),
        }

        Ok(())
    }

    async fn handle_server_message(
        &mut self,
        client: &mut Client<'_>,
        message: ServerMessage,
    ) -> Result<(), anyhow::Error> {
        match message {
            ServerMessage::ConnectionResponse(message) => {
                let packet = match message {
                    Ok(result) => Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Ok,
                        packet_sequence_id: result.packet_sequence_id,
                        pay_flags: 0xff,
                    }),
                    _ => Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Failed,
                        packet_sequence_id: 0,
                        pay_flags: 0,
                    }),
                };
                client.connection.write_packet(packet).await?;
            }
            ServerMessage::ReturnToCharacterSelect => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerReturnToCharacterSelect {}))
                    .await?;
            }
            ServerMessage::SelectCharacter(message) => match message {
                Ok(response) => {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerMoveServer {
                            login_token: response.login_token,
                            packet_codec_seed: response.packet_codec_seed,
                            ip: &response.ip,
                            port: response.port,
                        }))
                        .await?;
                }
                Err(_) => return Err(PacketError::InvalidPacket.into()),
            },
            ServerMessage::CharacterList(characters) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCharacterList { characters }))
                    .await?;
            }
            ServerMessage::CreateCharacter(message) => {
                let response = match message {
                    Ok(CreateCharacterResponse { character_slot }) => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::Ok,
                            is_platinum: character_slot >= 3,
                        })
                    }
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
            ServerMessage::DeleteCharacter(message) => {
                let packet = match message {
                    Ok(DeleteCharacterResponse { name, delete_time }) => {
                        Packet::from(&PacketServerDeleteCharacterReply {
                            seconds_until_delete: Some(
                                delete_time
                                    .map(|t| t.get_time_until_delete().as_secs())
                                    .unwrap_or(0) as u32,
                            ),
                            name: &name,
                        })
                    }
                    Err(DeleteCharacterError::Failed(name)) => {
                        Packet::from(&PacketServerDeleteCharacterReply {
                            seconds_until_delete: None,
                            name: &name,
                        })
                    }
                };
                client.connection.write_packet(packet).await?;
            }
            _ => panic!("Received unexpected server message for world server"),
        }
        Ok(())
    }
}

#[async_trait]
impl ProtocolServer for WorldServer {
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
