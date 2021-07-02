use async_trait::async_trait;
use num_traits::FromPrimitive;
use std::convert::TryFrom;
use tokio::sync::oneshot;

use crate::game::messages::{
    client::{
        Attack, ChangeEquipment, ClientMessage, GameConnectionRequest, JoinZoneRequest, Move,
        SetHotbarSlot,
    },
    server::{
        LocalChat, RemoveEntities, ServerMessage, SpawnEntityMonster, SpawnEntityNpc,
        UpdateEquipment, UpdateInventory, Whisper,
    },
};
use crate::protocol::{Client, Packet, ProtocolClient, ProtocolError};

mod common_packets;

mod client_packets;
mod server_packets;

use client_packets::*;
use server_packets::*;

pub struct GameClient {}

impl GameClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &self,
        client: &mut Client<'_>,
        packet: Packet,
    ) -> Result<(), ProtocolError> {
        match FromPrimitive::from_u16(packet.command) {
            Some(ClientPackets::ConnectRequest) => {
                let request = PacketClientConnectRequest::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::GameConnectionRequest(
                        GameConnectionRequest {
                            login_token: request.login_token,
                            password_md5: String::from(request.password_md5),
                            response_tx,
                        },
                    ))?;
                match response_rx.await? {
                    Ok(response) => {
                        client
                            .connection
                            .write_packet(Packet::from(&PacketConnectionReply {
                                result: ConnectResult::Ok,
                                packet_sequence_id: response.packet_sequence_id,
                                pay_flags: 0xff,
                            }))
                            .await?;

                        client
                            .connection
                            .write_packet(Packet::from(&PacketServerSelectCharacter {
                                character_info: &response.character_info,
                                position: &response.position,
                                equipment: &response.equipment,
                                basic_stats: &response.basic_stats,
                                level: &response.level,
                                skill_list: &response.skill_list,
                                hotbar: &response.hotbar,
                                health_points: &response.health_points,
                                mana_points: &response.mana_points,
                            }))
                            .await?;

                        client
                            .connection
                            .write_packet(Packet::from(&PacketServerCharacterInventory {
                                inventory: &response.inventory,
                                equipment: &response.equipment,
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
            Some(ClientPackets::JoinZone) => {
                let _request = PacketClientJoinZone::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::JoinZoneRequest(JoinZoneRequest {
                        response_tx,
                    }))?;
                let response = response_rx.await?;
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerJoinZone {
                        entity_id: response.entity_id,
                        level: &response.level,
                        team: &response.team,
                        health_points: &response.health_points,
                        mana_points: &response.mana_points,
                    }))
                    .await?;
            }
            Some(ClientPackets::Chat) => {
                let packet = PacketClientChat::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::Chat(String::from(packet.text)))?;
            }
            Some(ClientPackets::Move) => {
                let packet = PacketClientMove::try_from(&packet)?;
                client.client_message_tx.send(ClientMessage::Move(Move {
                    target_entity_id: packet.target_entity_id,
                    x: packet.x,
                    y: packet.y,
                    z: packet.z,
                }))?;
            }
            Some(ClientPackets::Attack) => {
                let packet = PacketClientAttack::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::Attack(Attack {
                        target_entity_id: packet.target_entity_id,
                    }))?;
            }
            Some(ClientPackets::SetHotbarSlot) => {
                let request = PacketClientSetHotbarSlot::try_from(&packet)?;
                let (response_tx, response_rx) = oneshot::channel();
                client
                    .client_message_tx
                    .send(ClientMessage::SetHotbarSlot(SetHotbarSlot {
                        slot_index: request.slot_index as usize,
                        slot: request.slot.clone(),
                        response_tx,
                    }))?;
                if response_rx.await?.is_ok() {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerSetHotbarSlot {
                            slot_index: request.slot_index,
                            slot: request.slot,
                        }))
                        .await?;
                }
            }
            Some(ClientPackets::ChangeEquipment) => {
                let PacketClientChangeEquipment {
                    equipment_index,
                    item_slot,
                } = PacketClientChangeEquipment::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ChangeEquipment(ChangeEquipment {
                        equipment_index,
                        item_slot,
                    }))?;
            }
            _ => println!("Unhandled packet 0x{:#03X}", packet.command),
        }
        Ok(())
    }

    async fn handle_server_message(
        &self,
        client: &mut Client<'_>,
        message: ServerMessage,
    ) -> Result<(), ProtocolError> {
        match message {
            ServerMessage::MoveEntity(message) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveEntity {
                        entity_id: message.entity_id,
                        target_entity_id: message.target_entity_id,
                        distance: message.distance,
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    }))
                    .await?;
            }
            ServerMessage::AttackEntity(message) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerAttackEntity {
                        entity_id: message.entity_id,
                        target_entity_id: message.target_entity_id,
                        distance: message.distance,
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    }))
                    .await?;
            }
            ServerMessage::DamageEntity(message) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerDamageEntity {
                        attacker_entity_id: message.attacker_entity_id,
                        defender_entity_id: message.defender_entity_id,
                        damage: message.damage,
                        is_killed: message.is_killed,
                    }))
                    .await?;
            }
            ServerMessage::StopMoveEntity(message) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerStopMoveEntity {
                        entity_id: message.entity_id,
                        x: message.x,
                        y: message.y,
                        z: message.z,
                    }))
                    .await?;
            }
            ServerMessage::Teleport(message) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerTeleport {
                        entity_id: message.entity_id,
                        zone_no: message.zone_no,
                        x: message.x,
                        y: message.y,
                        run_mode: message.run_mode,
                        ride_mode: message.ride_mode,
                    }))
                    .await?;
            }
            ServerMessage::LocalChat(LocalChat { entity_id, text }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLocalChat {
                        entity_id,
                        text: &text,
                    }))
                    .await?;
            }
            ServerMessage::Whisper(Whisper { from, text }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerWhisper {
                        from: &from,
                        text: &text,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityNpc(SpawnEntityNpc {
                entity_id,
                npc,
                direction,
                position,
                team,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityNpc {
                        entity_id,
                        npc: &npc,
                        direction: &direction,
                        position: &position,
                        team: &team,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityMonster(SpawnEntityMonster {
                entity_id,
                npc,
                position,
                team,
                health,
                destination,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityMonster {
                        entity_id,
                        npc: &npc,
                        position: &position,
                        team: &team,
                        health: &health,
                        destination: destination.as_ref(),
                    }))
                    .await?;
            }
            ServerMessage::RemoveEntities(RemoveEntities { entity_ids }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRemoveEntities {
                        entity_ids: &entity_ids,
                    }))
                    .await?;
            }
            ServerMessage::UpdateInventory(UpdateInventory { items }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateInventory { items: &items }))
                    .await?;
            }
            ServerMessage::UpdateEquipment(UpdateEquipment {
                entity_id,
                equipment_index,
                item,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateEquipment {
                        entity_id,
                        equipment_index,
                        item,
                        run_speed: None,
                    }))
                    .await?;
            }
        }
        Ok(())
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
