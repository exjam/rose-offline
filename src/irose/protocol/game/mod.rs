use async_trait::async_trait;
use log::warn;
use num_traits::FromPrimitive;
use std::convert::TryFrom;
use tokio::sync::oneshot;

use crate::{
    data::QuestTriggerHash,
    game::messages::{
        client::{
            Attack, ChangeEquipment, ClientMessage, GameConnectionRequest, JoinZoneRequest,
            LogoutRequest, Move, NpcStoreTransaction, PersonalStoreBuyItem, PickupItemDrop,
            QuestDelete, SetHotbarSlot,
        },
        server::{
            AnnounceChat, ApplySkillEffect, CastSkillSelf, CastSkillTargetEntity,
            CastSkillTargetPosition, LevelUpSkillResult, LocalChat, LogoutReply, MoveToggle,
            OpenPersonalStore, PartyMemberLeave, PersonalStoreTransactionCancelled,
            PersonalStoreTransactionResult, PersonalStoreTransactionSoldOut,
            PersonalStoreTransactionSuccess, PickupItemDropResult, QuestDeleteResult,
            QuestTriggerResult, RemoveEntities, ServerMessage, ShoutChat, SpawnEntityItemDrop,
            SpawnEntityMonster, SpawnEntityNpc, UpdateAbilityValue, UpdateBasicStat,
            UpdateEquipment, UpdateLevel, UpdateSpeed, UpdateStatusEffects, UpdateXpStamina,
            UseEmote, UseInventoryItem, UseItem, Whisper,
        },
    },
    protocol::{Client, Packet, ProtocolClient, ProtocolError},
};

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
                                experience_points: &response.experience_points,
                                skill_list: &response.skill_list,
                                hotbar: &response.hotbar,
                                health_points: &response.health_points,
                                mana_points: &response.mana_points,
                                stat_points: response.stat_points,
                                skill_points: response.skill_points,
                                union_membership: &response.union_membership,
                                stamina: response.stamina,
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
                            .write_packet(Packet::from(&PacketServerCharacterQuestData {
                                quest_state: &response.quest_state,
                            }))
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
                        experience_points: &response.experience_points,
                        team: &response.team,
                        health_points: &response.health_points,
                        mana_points: &response.mana_points,
                        world_ticks: response.world_ticks,
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
            Some(ClientPackets::ChangeAmmo) => {
                let PacketClientChangeAmmo {
                    ammo_index,
                    item_slot,
                } = PacketClientChangeAmmo::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ChangeAmmo(ammo_index, item_slot))?;
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
            Some(ClientPackets::IncreaseBasicStat) => {
                let PacketClientIncreaseBasicStat { basic_stat_type } =
                    PacketClientIncreaseBasicStat::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::IncreaseBasicStat(basic_stat_type))?;
            }
            Some(ClientPackets::PickupItemDrop) => {
                let packet = PacketClientPickupItemDrop::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PickupItemDrop(PickupItemDrop {
                        target_entity_id: packet.target_entity_id,
                    }))?;
            }
            Some(ClientPackets::LogoutRequest) => {
                client
                    .client_message_tx
                    .send(ClientMessage::LogoutRequest(LogoutRequest::Logout))?;
            }
            Some(ClientPackets::ReturnToCharacterSelectRequest) => {
                client.client_message_tx.send(ClientMessage::LogoutRequest(
                    LogoutRequest::ReturnToCharacterSelect,
                ))?;
            }
            Some(ClientPackets::ReviveRequest) => {
                let packet = PacketClientReviveRequest::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ReviveRequest(packet.revive_request_type))?;
            }
            Some(ClientPackets::QuestRequest) => {
                let packet = PacketClientQuestRequest::try_from(&packet)?;
                match packet.request_type {
                    PacketClientQuestRequestType::DoTrigger => {
                        client.client_message_tx.send(ClientMessage::QuestTrigger(
                            QuestTriggerHash::new(packet.quest_id),
                        ))?;
                    }
                    PacketClientQuestRequestType::DeleteQuest => {
                        client
                            .client_message_tx
                            .send(ClientMessage::QuestDelete(QuestDelete {
                                slot: packet.quest_slot as usize,
                                quest_id: packet.quest_id as usize,
                            }))?;
                    }
                }
            }
            Some(ClientPackets::PersonalStoreListItems) => {
                let packet = PacketClientPersonalStoreListItems::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PersonalStoreListItems(
                        packet.target_entity_id,
                    ))?;
            }
            Some(ClientPackets::PersonalStoreBuyItem) => {
                let packet = PacketClientPersonalStoreBuyItem::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PersonalStoreBuyItem(PersonalStoreBuyItem {
                        store_entity_id: packet.store_entity_id,
                        store_slot_index: packet.store_slot_index,
                        buy_item: packet.buy_item,
                    }))?;
            }
            Some(ClientPackets::DropItemFromInventory) => {
                let packet = PacketClientDropItemFromInventory::try_from(&packet)?;
                match packet {
                    PacketClientDropItemFromInventory::Item(item_slot, quantity) => {
                        client
                            .client_message_tx
                            .send(ClientMessage::DropItem(item_slot, quantity as usize))?;
                    }
                    PacketClientDropItemFromInventory::Money(quantity) => {
                        client
                            .client_message_tx
                            .send(ClientMessage::DropMoney(quantity as usize))?;
                    }
                }
            }
            Some(ClientPackets::UseItem) => {
                let packet = PacketClientUseItem::try_from(&packet)?;
                client.client_message_tx.send(ClientMessage::UseItem(
                    packet.item_slot,
                    packet.target_entity_id,
                ))?;
            }
            Some(ClientPackets::LevelUpSkill) => {
                let packet = PacketClientLevelUpSkill::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::LevelUpSkill(packet.skill_slot))?;
            }
            Some(ClientPackets::CastSkillSelf) => {
                let packet = PacketClientCastSkillSelf::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillSelf(packet.skill_slot))?;
            }
            Some(ClientPackets::CastSkillTargetEntity) => {
                let packet = PacketClientCastSkillTargetEntity::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillTargetEntity(
                        packet.skill_slot,
                        packet.target_entity_id,
                    ))?;
            }
            Some(ClientPackets::CastSkillTargetPosition) => {
                let packet = PacketClientCastSkillTargetPosition::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillTargetPosition(
                        packet.skill_slot,
                        packet.position,
                    ))?;
            }
            Some(ClientPackets::NpcStoreTransaction) => {
                let packet = PacketClientNpcStoreTransaction::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::NpcStoreTransaction(NpcStoreTransaction {
                        npc_entity_id: packet.npc_entity_id,
                        buy_items: packet.buy_items,
                        sell_items: packet.sell_items,
                    }))?;
            }
            Some(ClientPackets::MoveToggle) => {
                let packet = PacketClientMoveToggle::try_from(&packet)?;
                match packet.toggle_type {
                    PacketClientMoveToggleType::Run => {
                        client.client_message_tx.send(ClientMessage::RunToggle)?;
                    }
                    PacketClientMoveToggleType::Sit => {
                        client.client_message_tx.send(ClientMessage::SitToggle)?;
                    }
                    PacketClientMoveToggleType::Drive => {
                        client.client_message_tx.send(ClientMessage::DriveToggle)?;
                    }
                }
            }
            Some(ClientPackets::Emote) => {
                let packet = PacketClientEmote::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::UseEmote(packet.motion_id, packet.is_stop))?;
            }
            Some(ClientPackets::WarpGateRequest) => {
                let packet = PacketClientWarpGateRequest::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::WarpGateRequest(packet.warp_gate_id))?;
            }
            Some(ClientPackets::PartyRequest) => {
                let packet = PacketClientPartyRequest::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PartyRequest(packet.request))?;
            }
            Some(ClientPackets::PartyReply) => {
                let packet = PacketClientPartyReply::try_from(&packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PartyReply(packet.reply))?;
            }
            _ => warn!(
                "[GS] Unhandled packet [{:#03X}] {:02x?}",
                packet.command,
                &packet.data[..]
            ),
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
                        move_mode: message.move_mode,
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
                if message.from_skill.is_none() {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerDamageEntity {
                            attacker_entity_id: message.attacker_entity_id,
                            defender_entity_id: message.defender_entity_id,
                            damage: message.damage,
                            is_killed: message.is_killed,
                        }))
                        .await?;
                } else if let Some((skill_id, caster_intelligence)) = message.from_skill {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerApplySkillDamage {
                            entity_id: message.defender_entity_id,
                            caster_entity_id: message.attacker_entity_id,
                            caster_intelligence,
                            skill_id,
                            effect_success: [false, false],
                            damage: message.damage,
                            is_killed: message.is_killed,
                        }))
                        .await?;
                }
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
                        zone_id: message.zone_id,
                        x: message.x,
                        y: message.y,
                        run_mode: message.run_mode,
                        ride_mode: message.ride_mode,
                    }))
                    .await?;
            }
            ServerMessage::LocalChat(LocalChat {
                entity_id,
                ref text,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLocalChat { entity_id, text }))
                    .await?;
            }
            ServerMessage::ShoutChat(ShoutChat { ref name, ref text }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerShoutChat { name, text }))
                    .await?;
            }
            ServerMessage::AnnounceChat(AnnounceChat { ref name, ref text }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerAnnounceChat {
                        name: name.as_deref(),
                        text,
                    }))
                    .await?;
            }
            ServerMessage::Whisper(Whisper { ref from, ref text }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerWhisper { from, text }))
                    .await?;
            }
            ServerMessage::SpawnEntityCharacter(data) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityCharacter {
                        character_info: &data.character_info,
                        command: &data.command,
                        destination: data.destination.as_ref(),
                        entity_id: data.entity_id,
                        equipment: &data.equipment,
                        health: &data.health,
                        level: &data.level,
                        passive_attack_speed: data.passive_attack_speed,
                        position: &data.position,
                        status_effects: &data.status_effects,
                        move_mode: data.move_mode,
                        move_speed: data.move_speed,
                        target_entity_id: data.target_entity_id,
                        team: &data.team,
                        personal_store_info: &data.personal_store_info,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityItemDrop(SpawnEntityItemDrop {
                entity_id,
                ref dropped_item,
                ref position,
                ref remaining_time,
                owner_entity_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityItemDrop {
                        entity_id,
                        dropped_item,
                        position,
                        owner_entity_id,
                        remaining_time,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityNpc(SpawnEntityNpc {
                entity_id,
                ref npc,
                ref direction,
                ref position,
                ref team,
                ref health,
                destination,
                ref command,
                target_entity_id,
                move_mode,
                ref status_effects,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityNpc {
                        entity_id,
                        npc,
                        direction,
                        position,
                        team,
                        health,
                        destination: destination.as_ref(),
                        command,
                        target_entity_id,
                        move_mode,
                        status_effects,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityMonster(SpawnEntityMonster {
                entity_id,
                ref npc,
                ref position,
                ref team,
                ref health,
                destination,
                ref command,
                target_entity_id,
                move_mode,
                ref status_effects,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityMonster {
                        entity_id,
                        npc,
                        position,
                        team,
                        health,
                        destination: destination.as_ref(),
                        command,
                        target_entity_id,
                        move_mode,
                        status_effects,
                    }))
                    .await?;
            }
            ServerMessage::RemoveEntities(RemoveEntities { ref entity_ids }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRemoveEntities { entity_ids }))
                    .await?;
            }
            ServerMessage::UpdateAbilityValue(UpdateAbilityValue::RewardAdd(
                ability_type,
                value,
            )) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAbilityValue {
                        is_add: true,
                        ability_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::UpdateAbilityValue(UpdateAbilityValue::RewardSet(
                ability_type,
                value,
            )) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAbilityValue {
                        is_add: false,
                        ability_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::UpdateInventory(ref items, with_money) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateInventory {
                        items,
                        with_money,
                    }))
                    .await?;
            }
            ServerMessage::UpdateMoney(money) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateMoney { money }))
                    .await?;
            }
            ServerMessage::RewardItems(ref items) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRewardItems { items }))
                    .await?;
            }
            ServerMessage::RewardMoney(money) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRewardMoney { money }))
                    .await?;
            }
            ServerMessage::UpdateSpeed(UpdateSpeed {
                entity_id,
                run_speed,
                passive_attack_speed,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateSpeed {
                        entity_id,
                        run_speed,
                        passive_attack_speed,
                    }))
                    .await?;
            }
            ServerMessage::UpdateStatusEffects(UpdateStatusEffects {
                entity_id,
                ref status_effects,
                updated_hp,
                updated_mp,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateStatusEffects {
                        entity_id,
                        status_effects,
                        updated_hp,
                        updated_mp,
                    }))
                    .await?;
            }
            ServerMessage::UpdateAmmo(entity_id, ammo_index, item) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAmmo {
                        entity_id,
                        ammo_index,
                        item,
                    }))
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
            ServerMessage::UpdateLevel(UpdateLevel {
                entity_id,
                level,
                experience_points,
                stat_points,
                skill_points,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateLevel {
                        entity_id,
                        level,
                        experience_points,
                        stat_points,
                        skill_points,
                    }))
                    .await?;
            }
            ServerMessage::UpdateXpStamina(UpdateXpStamina {
                xp,
                stamina,
                source_entity_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateXpStamina {
                        xp,
                        stamina,
                        source_entity_id,
                    }))
                    .await?;
            }
            ServerMessage::UpdateBasicStat(UpdateBasicStat {
                basic_stat_type,
                value,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateBasicStat {
                        basic_stat_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::PickupItemDropResult(PickupItemDropResult {
                item_entity_id,
                result,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPickupItemDropResult {
                        item_entity_id,
                        result,
                    }))
                    .await?;
            }
            ServerMessage::LogoutReply(LogoutReply { result }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLogoutResult { result }))
                    .await?;
                if result.is_ok() {
                    return Err(ProtocolError::ServerInitiatedDisconnect);
                }
            }
            ServerMessage::QuestTriggerResult(QuestTriggerResult {
                success,
                trigger_hash,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerQuestResult {
                        result: if success {
                            PacketServerQuestResultType::TriggerSuccess
                        } else {
                            PacketServerQuestResultType::TriggerFailed
                        },
                        slot: 0,
                        quest_id: trigger_hash.hash,
                    }))
                    .await?;
            }
            ServerMessage::QuestDeleteResult(QuestDeleteResult {
                success,
                slot,
                quest_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerQuestResult {
                        result: if success {
                            PacketServerQuestResultType::DeleteSuccess
                        } else {
                            PacketServerQuestResultType::DeleteFailed
                        },
                        slot: slot as u8,
                        quest_id: quest_id as u32,
                    }))
                    .await?;
            }
            ServerMessage::LearnSkillResult(result) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLearnSkillResult { result }))
                    .await?;
            }
            ServerMessage::LevelUpSkillResult(LevelUpSkillResult {
                result,
                updated_skill_points,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLevelUpSkillResult {
                        result,
                        updated_skill_points,
                    }))
                    .await?;
            }
            ServerMessage::RunNpcDeathTrigger(npc_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRunNpcDeathTrigger { npc_id }))
                    .await?;
            }
            ServerMessage::OpenPersonalStore(OpenPersonalStore {
                entity_id,
                skin,
                title,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerOpenPersonalStore {
                        entity_id,
                        skin,
                        title: &title,
                    }))
                    .await?;
            }
            ServerMessage::PersonalStoreItemList(personal_store_item_list) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPersonalStoreItemList {
                        sell_items: &personal_store_item_list.sell_items,
                        buy_items: &personal_store_item_list.buy_items,
                    }))
                    .await?;
            }
            ServerMessage::PersonalStoreTransactionResult(result) => match result {
                PersonalStoreTransactionResult::Cancelled(PersonalStoreTransactionCancelled {
                    store_entity_id,
                }) => {
                    client
                        .connection
                        .write_packet(Packet::from(
                            &PacketServerPersonalStoreTransactionResult::Cancelled(store_entity_id),
                        ))
                        .await?;
                }
                PersonalStoreTransactionResult::SoldOut(PersonalStoreTransactionSoldOut {
                    store_entity_id,
                    store_slot_index,
                    item,
                }) => {
                    client
                        .connection
                        .write_packet(Packet::from(
                            &PacketServerPersonalStoreTransactionResult::SoldOut(
                                store_entity_id,
                                store_slot_index,
                                item,
                            ),
                        ))
                        .await?;
                }
                PersonalStoreTransactionResult::BoughtFromStore(
                    PersonalStoreTransactionSuccess {
                        store_entity_id,
                        store_slot_index,
                        store_slot_item,
                        money,
                        inventory_slot,
                        inventory_item,
                    },
                ) => {
                    client
                        .connection
                        .write_packet(Packet::from(
                            &PacketServerPersonalStoreTransactionUpdateMoneyAndInventory {
                                money,
                                slot: inventory_slot,
                                item: inventory_item,
                            },
                        ))
                        .await?;

                    client
                        .connection
                        .write_packet(Packet::from(
                            &PacketServerPersonalStoreTransactionResult::BoughtFromStore(
                                store_entity_id,
                                store_slot_index,
                                store_slot_item,
                            ),
                        ))
                        .await?;
                }
                PersonalStoreTransactionResult::NoMoreNeed(_) => todo!(),
                PersonalStoreTransactionResult::SoldToStore(_) => todo!(),
            },
            ServerMessage::UseItem(UseItem { entity_id, item }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseItem {
                        entity_id,
                        item,
                        inventory_slot: None,
                    }))
                    .await?;
            }
            ServerMessage::UseInventoryItem(UseInventoryItem {
                entity_id,
                item,
                inventory_slot,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseItem {
                        entity_id,
                        item,
                        inventory_slot: Some(inventory_slot),
                    }))
                    .await?;
            }
            ServerMessage::CastSkillSelf(CastSkillSelf {
                entity_id,
                skill_id,
                cast_motion_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCastSkillSelf {
                        entity_id,
                        skill_id,
                        cast_motion_id,
                    }))
                    .await?;
            }
            ServerMessage::CastSkillTargetEntity(CastSkillTargetEntity {
                entity_id,
                skill_id,
                target_entity_id,
                target_distance,
                target_position,
                cast_motion_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCastSkillTargetEntity {
                        entity_id,
                        skill_id,
                        target_entity_id,
                        target_distance,
                        target_position,
                        cast_motion_id,
                    }))
                    .await?;
            }
            ServerMessage::CastSkillTargetPosition(CastSkillTargetPosition {
                entity_id,
                skill_id,
                target_position,
                cast_motion_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCastSkillTargetPosition {
                        entity_id,
                        skill_id,
                        target_position,
                        cast_motion_id,
                    }))
                    .await?;
            }
            ServerMessage::StartCastingSkill(entity_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerStartCastingSkill { entity_id }))
                    .await?;
            }
            ServerMessage::ApplySkillEffect(ApplySkillEffect {
                entity_id,
                caster_entity_id,
                caster_intelligence,
                skill_id,
                effect_success,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerApplySkillEffect {
                        entity_id,
                        caster_entity_id,
                        caster_intelligence,
                        skill_id,
                        effect_success,
                    }))
                    .await?;
            }
            ServerMessage::CancelCastingSkill(entity_id, reason) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCancelCastingSkill {
                        entity_id,
                        reason,
                    }))
                    .await?;
            }
            ServerMessage::FinishCastingSkill(entity_id, skill_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerFinishCastingSkill {
                        entity_id,
                        skill_id,
                    }))
                    .await?;
            }
            ServerMessage::NpcStoreTransactionError(error) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerNpcStoreTransactionError {
                        error,
                    }))
                    .await?;
            }
            ServerMessage::MoveToggle(MoveToggle {
                entity_id,
                move_mode,
                run_speed,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveToggle {
                        entity_id,
                        move_mode,
                        run_speed,
                    }))
                    .await?;
            }
            ServerMessage::SitToggle(entity_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSitToggle { entity_id }))
                    .await?;
            }
            ServerMessage::UseEmote(UseEmote {
                entity_id,
                motion_id,
                is_stop,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseEmote {
                        entity_id,
                        motion_id,
                        is_stop,
                    }))
                    .await?;
            }
            ServerMessage::PartyRequest(party_request) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyRequest { party_request }))
                    .await?;
            }
            ServerMessage::PartyReply(party_reply) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply { party_reply }))
                    .await?;
            }
            ServerMessage::PartyMemberList(party_member_list) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMembers {
                        owner_character_id: party_member_list.owner_character_id,
                        party_members: &party_member_list.members,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberLeave(PartyMemberLeave {
                leaver_character_id,
                owner_character_id,
            }) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberLeave {
                        leaver_character_id,
                        owner_character_id,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberKicked(kicked_character_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberKicked {
                        kicked_character_id,
                    }))
                    .await?;
            }
            ServerMessage::PartyChangeOwner(client_entity_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyChangeOwner {
                        client_entity_id,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberDisconnect(character_id) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberDisconnect {
                        character_id,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberUpdateInfo(member_info) => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberUpdateInfo {
                        member_info,
                    }))
                    .await?;
            }
            // These messages are for World Server
            ServerMessage::ReturnToCharacterSelect => {
                panic!("Received unexpected server message for game server")
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
