use std::convert::TryFrom;

use crate::game::messages::{
    client::{ClientMessage, ConnectionRequest, GetInitialCharacterData, InitialCharacterData},
    server::ServerMessage,
};
use crate::protocol::{packet::Packet, Client, ProtocolClient, ProtocolError};
use async_trait::async_trait;
use num_traits::FromPrimitive;

mod client_packets;
mod server_packets;

use client_packets::*;
use server_packets::*;
use tokio::sync::oneshot;

pub struct GameClient {}

impl GameClient {
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
                match response_rx.await? {
                    Ok(result) => {
                        client
                            .connection
                            .write_packet(Packet::from(&PacketConnectionReply {
                                result: ConnectResult::Ok,
                                packet_sequence_id: result.packet_sequence_id,
                                pay_flags: 0xff,
                            }))
                            .await?;

                        let (response_tx, response_rx) = oneshot::channel();
                        // TODO: We're not getting a response here.
                        client
                            .client_message_tx
                            .send(ClientMessage::GetInitialCharacterData(
                                GetInitialCharacterData { response_tx },
                            ))?;

                        let character_data = response_rx.await?;

                        client
                            .connection
                            .write_packet(Packet::from(&PacketServerSelectCharacter {
                                character_info: &character_data.character_info,
                                position: &character_data.position,
                                equipment: &character_data.equipment,
                                basic_stats: &character_data.basic_stats,
                                level: &character_data.level,
                            }))
                            .await?;

                        client
                            .connection
                            .write_packet(Packet::from(&PacketServerCharacterInventory {
                                inventory: &character_data.inventory,
                                equipment: &character_data.equipment,
                            }))
                            .await?;

                        client
                            .connection
                            .write_packet(Packet::from(&PacketServerCharacterQuestData {}))
                            .await?;
                    }
                    Err(_) => {
                        client
                            .connection
                            .write_packet(Packet::from(&PacketConnectionReply {
                                result: ConnectResult::Failed,
                                packet_sequence_id: 0,
                                pay_flags: 0,
                            }))
                            .await?;
                    }
                };
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
                panic!("Unimplemented message for irose game server!")
            }
        }
    }
}

#[async_trait]
impl ProtocolClient for GameClient {
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
