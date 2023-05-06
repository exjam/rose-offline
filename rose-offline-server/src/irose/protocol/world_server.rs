use async_trait::async_trait;
use log::warn;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::{
    data::Password,
    messages::{
        client::ClientMessage,
        server::{CreateCharacterError, ServerMessage},
    },
};
use rose_network_common::{Packet, PacketError};
use rose_network_irose::{world_client_packets::*, world_server_packets::*};

use crate::{
    implement_protocol_server,
    protocol::{Client, ProtocolServer, ProtocolServerError},
};

pub struct WorldServer;

impl WorldServer {
    pub fn new() -> Self {
        Self {}
    }

    fn handle_packet(
        &mut self,
        client: &mut Client<'_>,
        packet: &Packet,
    ) -> Result<(), anyhow::Error> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::ConnectRequest) => {
                let request = PacketClientConnectRequest::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ConnectionRequest {
                        login_token: request.login_token,
                        password: Password::Md5(request.password_md5.into()),
                    })?;
            }
            Some(ClientPackets::CharacterListRequest) => {
                client
                    .client_message_tx
                    .send(ClientMessage::GetCharacterList)?;
            }
            Some(ClientPackets::CreateCharacter) => {
                let request = PacketClientCreateCharacter::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CreateCharacter {
                        gender: request.gender,
                        birth_stone: request.birth_stone as i32,
                        hair: request.hair as i32,
                        face: request.face as i32,
                        start_point: request.start_point as i32,
                        hair_color: 1,
                        weapon_type: 0,
                        name: String::from(request.name),
                    })?;
            }
            Some(ClientPackets::DeleteCharacter) => {
                let request = PacketClientDeleteCharacter::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::DeleteCharacter {
                        slot: request.slot,
                        name: String::from(request.name),
                        is_delete: request.is_delete,
                    })?;
            }
            Some(ClientPackets::SelectCharacter) => {
                let request = PacketClientSelectCharacter::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::SelectCharacter {
                        slot: request.slot,
                        name: String::from(request.name),
                    })?;
            }
            Some(ClientPackets::ClanCommand) => match PacketClientClanCommand::try_from(packet)? {
                PacketClientClanCommand::GetMemberList => client
                    .client_message_tx
                    .send(ClientMessage::ClanGetMemberList)?,
                PacketClientClanCommand::UpdateLevelAndJob { .. } => {
                    // Ignore this, we do not need to rely on client reporting of level / job
                }
            },
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
            ServerMessage::ConnectionRequestSuccess { packet_sequence_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Ok,
                        packet_sequence_id,
                        pay_flags: 0xff,
                    }))
                    .await?;
            }
            ServerMessage::ConnectionRequestError { .. } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketConnectionReply {
                        result: ConnectResult::Failed,
                        packet_sequence_id: 0,
                        pay_flags: 0,
                    }))
                    .await?;
            }
            ServerMessage::ReturnToCharacterSelect => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerReturnToCharacterSelect {}))
                    .await?;
            }
            ServerMessage::SelectCharacterSuccess {
                login_token,
                packet_codec_seed,
                ref ip,
                port,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveServer {
                        login_token,
                        packet_codec_seed,
                        ip,
                        port,
                    }))
                    .await?;
            }
            ServerMessage::SelectCharacterError => {
                return Err(PacketError::InvalidPacket.into());
            }
            ServerMessage::CharacterList { character_list } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCharacterList {
                        characters: character_list,
                    }))
                    .await?;
            }
            ServerMessage::CreateCharacterSuccess { character_slot } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCreateCharacterReply {
                        result: CreateCharacterResult::Ok,
                        is_platinum: character_slot >= 3,
                    }))
                    .await?;
            }
            ServerMessage::CreateCharacterError { error } => {
                let packet = match error {
                    CreateCharacterError::Failed => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::Failed,
                            is_platinum: false,
                        })
                    }
                    CreateCharacterError::AlreadyExists => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::NameAlreadyExists,
                            is_platinum: false,
                        })
                    }
                    CreateCharacterError::InvalidValue => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::InvalidValue,
                            is_platinum: false,
                        })
                    }
                    CreateCharacterError::NoMoreSlots => {
                        Packet::from(&PacketServerCreateCharacterReply {
                            result: CreateCharacterResult::NoMoreSlots,
                            is_platinum: false,
                        })
                    }
                };

                client.connection.write_packet(packet).await?;
            }
            ServerMessage::DeleteCharacterStart { name, delete_time } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerDeleteCharacterReply {
                        seconds_until_delete: Some(
                            delete_time.get_time_until_delete().as_secs() as u32
                        ),
                        name: &name,
                    }))
                    .await?;
            }
            ServerMessage::DeleteCharacterCancel { name } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerDeleteCharacterReply {
                        seconds_until_delete: None,
                        name: &name,
                    }))
                    .await?;
            }
            ServerMessage::DeleteCharacterError { name } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerDeleteCharacterReply {
                        seconds_until_delete: None,
                        name: &name,
                    }))
                    .await?;
            }
            _ => panic!("Received unexpected server message for world server"),
        }
        Ok(())
    }
}

implement_protocol_server! { WorldServer }
