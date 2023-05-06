use async_trait::async_trait;
use log::warn;
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use rose_data::QuestTriggerHash;
use rose_game_common::{
    components::MoveMode,
    data::Password,
    messages::{client::ClientMessage, server::ServerMessage},
};
use rose_network_common::Packet;
use rose_network_irose::{game_client_packets::*, game_server_packets::*};

use crate::{
    implement_protocol_server,
    protocol::{Client, ProtocolServer, ProtocolServerError},
};

pub struct GameServer;

impl GameServer {
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
                    .send(ClientMessage::GameConnectionRequest {
                        login_token: request.login_token,
                        password: Password::Md5(request.password_md5.into()),
                    })?;
            }
            Some(ClientPackets::JoinZone) => {
                let _request = PacketClientJoinZone::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::JoinZoneRequest)?;
            }
            Some(ClientPackets::Chat) => {
                let packet = PacketClientChat::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::Chat {
                    text: String::from(packet.text),
                })?;
            }
            Some(ClientPackets::Move) => {
                let packet = PacketClientMove::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::Move {
                    target_entity_id: packet.target_entity_id,
                    x: packet.x,
                    y: packet.y,
                    z: packet.z,
                })?;
            }
            Some(ClientPackets::Attack) => {
                let packet = PacketClientAttack::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::Attack {
                    target_entity_id: packet.target_entity_id,
                })?;
            }
            Some(ClientPackets::SetHotbarSlot) => {
                let request = PacketClientSetHotbarSlot::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::SetHotbarSlot {
                        slot_index: request.slot_index,
                        slot: request.slot,
                    })?;
            }
            Some(ClientPackets::ChangeAmmo) => {
                let PacketClientChangeAmmo {
                    ammo_index,
                    item_slot,
                } = PacketClientChangeAmmo::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::ChangeAmmo {
                    ammo_index,
                    item_slot,
                })?;
            }
            Some(ClientPackets::ChangeEquipment) => {
                let PacketClientChangeEquipment {
                    equipment_index,
                    item_slot,
                } = PacketClientChangeEquipment::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ChangeEquipment {
                        equipment_index,
                        item_slot,
                    })?;
            }
            Some(ClientPackets::ChangeVehiclePart) => {
                let PacketClientChangeVehiclePart {
                    vehicle_part_index,
                    item_slot,
                } = PacketClientChangeVehiclePart::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::ChangeVehiclePart {
                        vehicle_part_index,
                        item_slot,
                    })?;
            }
            Some(ClientPackets::IncreaseBasicStat) => {
                let PacketClientIncreaseBasicStat { basic_stat_type } =
                    PacketClientIncreaseBasicStat::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::IncreaseBasicStat { basic_stat_type })?;
            }
            Some(ClientPackets::PickupItemDrop) => {
                let packet = PacketClientPickupItemDrop::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PickupItemDrop {
                        target_entity_id: packet.target_entity_id,
                    })?;
            }
            Some(ClientPackets::LogoutRequest) => {
                client.client_message_tx.send(ClientMessage::Logout)?;
            }
            Some(ClientPackets::ReturnToCharacterSelectRequest) => {
                client
                    .client_message_tx
                    .send(ClientMessage::ReturnToCharacterSelect)?;
            }
            Some(ClientPackets::ReviveRequest) => {
                let message = match PacketClientReviveRequest::try_from(packet)? {
                    PacketClientReviveRequest::CurrentZone => ClientMessage::ReviveCurrentZone,
                    PacketClientReviveRequest::SaveZone => ClientMessage::ReviveSaveZone,
                };
                client.client_message_tx.send(message)?;
            }
            Some(ClientPackets::SetReviveZone) => {
                client
                    .client_message_tx
                    .send(ClientMessage::SetReviveSaveZone)?;
            }
            Some(ClientPackets::QuestRequest) => {
                let packet = PacketClientQuestRequest::try_from(packet)?;
                match packet.request_type {
                    PacketClientQuestRequestType::DoTrigger => {
                        client.client_message_tx.send(ClientMessage::QuestTrigger {
                            trigger: QuestTriggerHash::new(packet.quest_id),
                        })?;
                    }
                    PacketClientQuestRequestType::DeleteQuest => {
                        client.client_message_tx.send(ClientMessage::QuestDelete {
                            slot: packet.quest_slot as usize,
                            quest_id: packet.quest_id as usize,
                        })?;
                    }
                }
            }
            Some(ClientPackets::PersonalStoreListItems) => {
                let packet = PacketClientPersonalStoreListItems::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PersonalStoreListItems {
                        store_entity_id: packet.target_entity_id,
                    })?;
            }
            Some(ClientPackets::PersonalStoreBuyItem) => {
                let packet = PacketClientPersonalStoreBuyItem::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PersonalStoreBuyItem {
                        store_entity_id: packet.store_entity_id,
                        store_slot_index: packet.store_slot_index,
                        buy_item: packet.buy_item,
                    })?;
            }
            Some(ClientPackets::DropItemFromInventory) => {
                let packet = PacketClientDropItemFromInventory::try_from(packet)?;
                match packet {
                    PacketClientDropItemFromInventory::Item(item_slot, quantity) => {
                        client.client_message_tx.send(ClientMessage::DropItem {
                            item_slot,
                            quantity: quantity as usize,
                        })?;
                    }
                    PacketClientDropItemFromInventory::Money(quantity) => {
                        client.client_message_tx.send(ClientMessage::DropMoney {
                            quantity: quantity as usize,
                        })?;
                    }
                }
            }
            Some(ClientPackets::UseItem) => {
                let packet = PacketClientUseItem::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::UseItem {
                    item_slot: packet.item_slot,
                    target_entity_id: packet.target_entity_id,
                })?;
            }
            Some(ClientPackets::LevelUpSkill) => {
                let packet = PacketClientLevelUpSkill::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::LevelUpSkill {
                    skill_slot: packet.skill_slot,
                })?;
            }
            Some(ClientPackets::CastSkillSelf) => {
                let packet = PacketClientCastSkillSelf::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillSelf {
                        skill_slot: packet.skill_slot,
                    })?;
            }
            Some(ClientPackets::CastSkillTargetEntity) => {
                let packet = PacketClientCastSkillTargetEntity::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillTargetEntity {
                        skill_slot: packet.skill_slot,
                        target_entity_id: packet.target_entity_id,
                    })?;
            }
            Some(ClientPackets::CastSkillTargetPosition) => {
                let packet = PacketClientCastSkillTargetPosition::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::CastSkillTargetPosition {
                        skill_slot: packet.skill_slot,
                        position: packet.position,
                    })?;
            }
            Some(ClientPackets::NpcStoreTransaction) => {
                let packet = PacketClientNpcStoreTransaction::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::NpcStoreTransaction {
                        npc_entity_id: packet.npc_entity_id,
                        buy_items: packet.buy_items,
                        sell_items: packet.sell_items,
                    })?;
            }
            Some(ClientPackets::MoveToggle) => {
                let packet = PacketClientMoveToggle::try_from(packet)?;
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
                let packet = PacketClientEmote::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::UseEmote {
                    motion_id: packet.motion_id,
                    is_stop: packet.is_stop,
                })?;
            }
            Some(ClientPackets::WarpGateRequest) => {
                let packet = PacketClientWarpGateRequest::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::WarpGateRequest {
                        warp_gate_id: packet.warp_gate_id,
                    })?;
            }
            Some(ClientPackets::PartyRequest) => {
                let message = match PacketClientPartyRequest::try_from(packet)? {
                    PacketClientPartyRequest::Create(invited_entity_id) => {
                        ClientMessage::PartyCreate { invited_entity_id }
                    }
                    PacketClientPartyRequest::Invite(invited_entity_id) => {
                        ClientMessage::PartyInvite { invited_entity_id }
                    }
                    PacketClientPartyRequest::Leave => ClientMessage::PartyLeave,
                    PacketClientPartyRequest::ChangeOwner(new_owner_entity_id) => {
                        ClientMessage::PartyChangeOwner {
                            new_owner_entity_id,
                        }
                    }
                    PacketClientPartyRequest::Kick(character_id) => {
                        ClientMessage::PartyKick { character_id }
                    }
                };

                client.client_message_tx.send(message)?;
            }
            Some(ClientPackets::PartyReply) => {
                let message = match PacketClientPartyReply::try_from(packet)? {
                    PacketClientPartyReply::AcceptCreate(owner_entity_id) => {
                        ClientMessage::PartyAcceptCreateInvite { owner_entity_id }
                    }
                    PacketClientPartyReply::AcceptJoin(owner_entity_id) => {
                        ClientMessage::PartyAcceptJoinInvite { owner_entity_id }
                    }
                    PacketClientPartyReply::Reject(reason, owner_entity_id) => {
                        ClientMessage::PartyRejectInvite {
                            reason,
                            owner_entity_id,
                        }
                    }
                };

                client.client_message_tx.send(message)?;
            }
            Some(ClientPackets::PartyUpdateRules) => {
                let message = PacketClientPartyUpdateRules::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::PartyUpdateRules {
                        item_sharing: message.item_sharing,
                        xp_sharing: message.xp_sharing,
                    })?;
            }
            Some(ClientPackets::MoveCollision) => {
                let message = PacketClientMoveCollision::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::MoveCollision {
                        position: message.position,
                    })?;
            }
            Some(ClientPackets::CraftItem) => {
                let packet = PacketClientCraftItem::try_from(packet)?;
                let message = match packet {
                    PacketClientCraftItem::InsertGem {
                        equipment_index,
                        item_slot,
                    } => ClientMessage::CraftInsertGem {
                        equipment_index,
                        item_slot,
                    },
                    PacketClientCraftItem::SkillDisassemble {
                        skill_slot,
                        item_slot,
                    } => ClientMessage::CraftSkillDisassemble {
                        skill_slot,
                        item_slot,
                    },
                    PacketClientCraftItem::NpcDisassemble {
                        npc_entity_id,
                        item_slot,
                    } => ClientMessage::CraftNpcDisassemble {
                        npc_entity_id,
                        item_slot,
                    },
                    PacketClientCraftItem::SkillUpgradeItem {
                        skill_slot,
                        item_slot,
                        ingredients,
                    } => ClientMessage::CraftSkillUpgradeItem {
                        skill_slot,
                        item_slot,
                        ingredients,
                    },
                    PacketClientCraftItem::NpcUpgradeItem {
                        npc_entity_id,
                        item_slot,
                        ingredients,
                    } => ClientMessage::CraftNpcUpgradeItem {
                        npc_entity_id,
                        item_slot,
                        ingredients,
                    },
                };
                client.client_message_tx.send(message)?;
            }
            Some(ClientPackets::BankOpen) => {
                let _ = PacketClientBankOpen::try_from(packet)?;
                client.client_message_tx.send(ClientMessage::BankOpen)?;
            }
            Some(ClientPackets::BankMoveItem) => {
                let message = match PacketClientBankMoveItem::try_from(packet)? {
                    PacketClientBankMoveItem::Deposit {
                        item_slot,
                        item,
                        is_premium,
                    } => ClientMessage::BankDepositItem {
                        item_slot,
                        item,
                        is_premium,
                    },
                    PacketClientBankMoveItem::Withdraw {
                        bank_slot,
                        item,
                        is_premium,
                    } => ClientMessage::BankWithdrawItem {
                        bank_slot,
                        item,
                        is_premium,
                    },
                };

                client.client_message_tx.send(message)?;
            }
            Some(ClientPackets::RepairItemUsingItem) => {
                let packet = PacketClientRepairItemUsingItem::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::RepairItemUsingItem {
                        use_item_slot: packet.use_item_slot,
                        item_slot: packet.item_slot,
                    })?;
            }
            Some(ClientPackets::RepairItemUsingNpc) => {
                let packet = PacketClientRepairItemUsingNpc::try_from(packet)?;
                client
                    .client_message_tx
                    .send(ClientMessage::RepairItemUsingNpc {
                        npc_entity_id: packet.npc_entity_id,
                        item_slot: packet.item_slot,
                    })?;
            }
            Some(ClientPackets::ClanCommand) => match PacketClientClanCommand::try_from(packet)? {
                PacketClientClanCommand::Create {
                    mark,
                    name,
                    description,
                } => client.client_message_tx.send(ClientMessage::ClanCreate {
                    name,
                    description,
                    mark,
                })?,
            },
            _ => warn!(
                "[GS] Unhandled packet [{:#03X}] {:02x?}",
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
            ServerMessage::CharacterData { data } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSelectCharacter {
                        character_info: data.character_info,
                        position: data.position,
                        zone_id: data.zone_id,
                        equipment: data.equipment,
                        basic_stats: data.basic_stats,
                        level: data.level,
                        experience_points: data.experience_points,
                        skill_list: data.skill_list,
                        hotbar: data.hotbar,
                        health_points: data.health_points,
                        mana_points: data.mana_points,
                        stat_points: data.stat_points,
                        skill_points: data.skill_points,
                        union_membership: data.union_membership,
                        stamina: data.stamina,
                    }))
                    .await?;
            }
            ServerMessage::CharacterDataItems { data } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCharacterInventory {
                        inventory: data.inventory,
                        equipment: data.equipment,
                    }))
                    .await?;
            }
            ServerMessage::CharacterDataQuest { quest_state } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCharacterQuestData {
                        quest_state: *quest_state,
                    }))
                    .await?;
            }
            ServerMessage::JoinZone {
                entity_id,
                experience_points,
                team,
                health_points,
                mana_points,
                world_ticks,
                craft_rate,
                world_price_rate,
                item_price_rate,
                town_price_rate,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerJoinZone {
                        entity_id,
                        experience_points,
                        team,
                        health_points,
                        mana_points,
                        world_ticks,
                        craft_rate,
                        world_price_rate,
                        item_price_rate,
                        town_price_rate,
                    }))
                    .await?;
            }
            ServerMessage::MoveEntity {
                entity_id,
                target_entity_id,
                distance,
                x,
                y,
                z,
                move_mode,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveEntity {
                        entity_id,
                        target_entity_id,
                        distance,
                        x,
                        y,
                        z,
                        move_mode,
                    }))
                    .await?;
            }
            ServerMessage::AttackEntity {
                entity_id,
                target_entity_id,
                distance,
                x,
                y,
                z,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerAttackEntity {
                        entity_id,
                        target_entity_id,
                        distance,
                        x,
                        y,
                        z,
                    }))
                    .await?;
            }
            ServerMessage::DamageEntity {
                attacker_entity_id,
                defender_entity_id,
                damage,
                is_killed,
                is_immediate,
                from_skill,
            } => match from_skill {
                None => {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerDamageEntity {
                            attacker_entity_id,
                            defender_entity_id,
                            damage,
                            is_killed,
                            is_immediate,
                        }))
                        .await?;
                }
                Some((skill_id, caster_intelligence)) => {
                    client
                        .connection
                        .write_packet(Packet::from(&PacketServerApplySkillDamage {
                            entity_id: defender_entity_id,
                            caster_entity_id: attacker_entity_id,
                            caster_intelligence,
                            skill_id,
                            effect_success: [false, false],
                            damage,
                            is_killed,
                            is_immediate,
                        }))
                        .await?;
                }
            },
            ServerMessage::StopMoveEntity { entity_id, x, y, z } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerStopMoveEntity {
                        entity_id,
                        x,
                        y,
                        z,
                    }))
                    .await?;
            }
            ServerMessage::Teleport {
                entity_id,
                zone_id,
                x,
                y,
                run_mode,
                ride_mode,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerTeleport {
                        entity_id,
                        zone_id,
                        x,
                        y,
                        run_mode,
                        ride_mode,
                    }))
                    .await?;
            }
            ServerMessage::LocalChat {
                entity_id,
                ref text,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLocalChat { entity_id, text }))
                    .await?;
            }
            ServerMessage::ShoutChat { ref name, ref text } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerShoutChat { name, text }))
                    .await?;
            }
            ServerMessage::AnnounceChat { ref name, ref text } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerAnnounceChat {
                        name: name.as_deref(),
                        text,
                    }))
                    .await?;
            }
            ServerMessage::Whisper { ref from, ref text } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerWhisper { from, text }))
                    .await?;
            }
            ServerMessage::SpawnEntityCharacter { data } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityCharacter {
                        character_info: data.character_info,
                        spawn_command_state: data.spawn_command_state,
                        entity_id: data.entity_id,
                        equipment: data.equipment,
                        health: data.health,
                        level: data.level,
                        passive_attack_speed: data.passive_attack_speed,
                        position: data.position,
                        status_effects: data.status_effects,
                        move_mode: data.move_mode,
                        move_speed: data.move_speed,
                        team: data.team,
                        personal_store_info: data.personal_store_info,
                        clan_membership: data.clan_membership,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityItemDrop {
                entity_id,
                dropped_item,
                position,
                remaining_time,
                owner_entity_id,
            } => {
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
            ServerMessage::SpawnEntityNpc {
                entity_id,
                npc,
                direction,
                position,
                team,
                health,
                spawn_command_state: command,
                move_mode,
                status_effects,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityNpc {
                        entity_id,
                        npc,
                        direction,
                        position,
                        team,
                        health,
                        spawn_command_state: command,
                        move_mode,
                        status_effects,
                    }))
                    .await?;
            }
            ServerMessage::SpawnEntityMonster {
                entity_id,
                npc,
                position,
                team,
                health,
                spawn_command_state: command,
                move_mode,
                status_effects,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSpawnEntityMonster {
                        entity_id,
                        npc,
                        position,
                        team,
                        health,
                        spawn_command_state: command,
                        move_mode,
                        status_effects,
                    }))
                    .await?;
            }
            ServerMessage::RemoveEntities { entity_ids } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRemoveEntities { entity_ids }))
                    .await?;
            }
            ServerMessage::UpdateAbilityValueAdd {
                ability_type,
                value,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAbilityValue {
                        is_add: true,
                        ability_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::UpdateAbilityValueSet {
                ability_type,
                value,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAbilityValue {
                        is_add: false,
                        ability_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::UpdateInventory { items, money } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateInventory {
                        items,
                        with_money: money,
                    }))
                    .await?;
            }
            ServerMessage::UpdateMoney { money } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateMoney { money }))
                    .await?;
            }
            ServerMessage::RewardItems { items } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRewardItems { items }))
                    .await?;
            }
            ServerMessage::RewardMoney { money } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRewardMoney { money }))
                    .await?;
            }
            ServerMessage::UpdateSpeed {
                entity_id,
                run_speed,
                passive_attack_speed,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateSpeed {
                        entity_id,
                        run_speed,
                        passive_attack_speed,
                    }))
                    .await?;
            }
            ServerMessage::UpdateStatusEffects {
                entity_id,
                status_effects,
                updated_values,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateStatusEffects {
                        entity_id,
                        status_effects,
                        updated_values,
                    }))
                    .await?;
            }
            ServerMessage::UpdateAmmo {
                entity_id,
                ammo_index,
                item,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateAmmo {
                        entity_id,
                        ammo_index,
                        item,
                    }))
                    .await?;
            }
            ServerMessage::UpdateEquipment {
                entity_id,
                equipment_index,
                item,
            } => {
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
            ServerMessage::UpdateVehiclePart {
                entity_id,
                vehicle_part_index,
                item,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateVehiclePart {
                        entity_id,
                        vehicle_part_index,
                        item,
                        run_speed: None,
                    }))
                    .await?;
            }
            ServerMessage::UpdateItemLife { item_slot, life } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateItemLife {
                        item_slot,
                        life,
                    }))
                    .await?;
            }
            ServerMessage::UpdateLevel {
                entity_id,
                level,
                experience_points,
                stat_points,
                skill_points,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateLevel {
                        entity_id,
                        update_values: Some((level, experience_points, stat_points, skill_points)),
                    }))
                    .await?;
            }
            ServerMessage::LevelUpEntity { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateLevel {
                        entity_id,
                        update_values: None,
                    }))
                    .await?;
            }
            ServerMessage::UpdateXpStamina {
                xp,
                stamina,
                source_entity_id,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateXpStamina {
                        xp,
                        stamina,
                        source_entity_id,
                    }))
                    .await?;
            }
            ServerMessage::UpdateBasicStat {
                basic_stat_type,
                value,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUpdateBasicStat {
                        basic_stat_type,
                        value,
                    }))
                    .await?;
            }
            ServerMessage::PickupDropItem {
                drop_entity_id,
                item_slot,
                item,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPickupItemDropResult::Item {
                        drop_entity_id,
                        item_slot,
                        item,
                    }))
                    .await?;
            }
            ServerMessage::PickupDropMoney {
                drop_entity_id,
                money,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPickupItemDropResult::Money {
                        drop_entity_id,
                        money,
                    }))
                    .await?;
            }
            ServerMessage::PickupDropError {
                drop_entity_id,
                error,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPickupItemDropResult::Error {
                        drop_entity_id,
                        error,
                    }))
                    .await?;
            }
            ServerMessage::LogoutSuccess => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLogoutResult { result: Ok(()) }))
                    .await?;
                return Err(ProtocolServerError::ServerInitiatedDisconnect.into());
            }
            ServerMessage::LogoutFailed { wait_duration } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLogoutResult {
                        result: Err(wait_duration),
                    }))
                    .await?;
            }
            ServerMessage::QuestTriggerResult {
                success,
                trigger_hash,
            } => {
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
            ServerMessage::QuestDeleteResult {
                success,
                slot,
                quest_id,
            } => {
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
            ServerMessage::LearnSkillSuccess {
                skill_slot,
                skill_id,
                updated_skill_points,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLearnSkillResult::Success {
                        skill_slot,
                        skill_id,
                        updated_skill_points,
                    }))
                    .await?;
            }
            ServerMessage::LearnSkillError { error } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLearnSkillResult::Error { error }))
                    .await?;
            }
            ServerMessage::LevelUpSkillSuccess {
                skill_slot,
                skill_id,
                skill_points,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLevelUpSkillResult::Success {
                        skill_slot,
                        skill_id,
                        skill_points,
                    }))
                    .await?;
            }
            ServerMessage::LevelUpSkillError {
                error,
                skill_points,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerLevelUpSkillResult::Error {
                        error,
                        skill_points,
                    }))
                    .await?;
            }
            ServerMessage::RunNpcDeathTrigger { npc_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRunNpcDeathTrigger { npc_id }))
                    .await?;
            }
            ServerMessage::OpenPersonalStore {
                entity_id,
                skin,
                title,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerOpenPersonalStore {
                        entity_id,
                        skin,
                        title: &title,
                    }))
                    .await?;
            }
            ServerMessage::ClosePersonalStore { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerClosePersonalStore { entity_id }))
                    .await?;
            }
            ServerMessage::PersonalStoreItemList {
                sell_items,
                buy_items,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPersonalStoreItemList {
                        sell_items,
                        buy_items,
                    }))
                    .await?;
            }
            ServerMessage::PersonalStoreTransaction {
                status,
                store_entity_id,
                update_store,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPersonalStoreTransactionResult {
                        status,
                        store_entity_id,
                        update_store_items: update_store,
                    }))
                    .await?;
            }
            ServerMessage::PersonalStoreTransactionUpdateInventory { money, items } => {
                client
                    .connection
                    .write_packet(Packet::from(
                        &PacketServerPersonalStoreTransactionUpdateMoneyAndInventory {
                            money,
                            items,
                        },
                    ))
                    .await?;
            }
            ServerMessage::UseItem { entity_id, item } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseItem {
                        entity_id,
                        item,
                        inventory_slot: None,
                    }))
                    .await?;
            }
            ServerMessage::UseInventoryItem {
                entity_id,
                item,
                inventory_slot,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseItem {
                        entity_id,
                        item,
                        inventory_slot: Some(inventory_slot),
                    }))
                    .await?;
            }
            ServerMessage::CastSkillSelf {
                entity_id,
                skill_id,
                cast_motion_id,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCastSkillSelf {
                        entity_id,
                        skill_id,
                        cast_motion_id,
                    }))
                    .await?;
            }
            ServerMessage::CastSkillTargetEntity {
                entity_id,
                skill_id,
                target_entity_id,
                target_distance,
                target_position,
                cast_motion_id,
            } => {
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
            ServerMessage::CastSkillTargetPosition {
                entity_id,
                skill_id,
                target_position,
                cast_motion_id,
            } => {
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
            ServerMessage::StartCastingSkill { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerStartCastingSkill { entity_id }))
                    .await?;
            }
            ServerMessage::ApplySkillEffect {
                entity_id,
                caster_entity_id,
                caster_intelligence,
                skill_id,
                effect_success,
            } => {
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
            ServerMessage::CancelCastingSkill { entity_id, reason } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCancelCastingSkill {
                        entity_id,
                        reason,
                    }))
                    .await?;
            }
            ServerMessage::FinishCastingSkill {
                entity_id,
                skill_id,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerFinishCastingSkill {
                        entity_id,
                        skill_id,
                    }))
                    .await?;
            }
            ServerMessage::NpcStoreTransactionError { error } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerNpcStoreTransactionError {
                        error,
                    }))
                    .await?;
            }
            ServerMessage::MoveToggle {
                entity_id,
                move_mode,
                run_speed,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveToggle {
                        entity_id,
                        move_toggle_type: match move_mode {
                            MoveMode::Walk => PacketServerMoveToggleType::Walk,
                            MoveMode::Run => PacketServerMoveToggleType::Run,
                            MoveMode::Drive => PacketServerMoveToggleType::Drive,
                        },
                        run_speed,
                    }))
                    .await?;
            }
            ServerMessage::SitToggle { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerMoveToggle {
                        entity_id,
                        move_toggle_type: PacketServerMoveToggleType::Sit,
                        run_speed: None,
                    }))
                    .await?;
            }
            ServerMessage::UseEmote {
                entity_id,
                motion_id,
                is_stop,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerUseEmote {
                        entity_id,
                        motion_id,
                        is_stop,
                    }))
                    .await?;
            }
            ServerMessage::PartyCreate { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyRequest::Create(entity_id)))
                    .await?;
            }
            ServerMessage::PartyInvite { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyRequest::Invite(entity_id)))
                    .await?;
            }
            ServerMessage::PartyAcceptCreate { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::AcceptCreate(
                        entity_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyAcceptInvite { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::AcceptInvite(
                        entity_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyRejectInvite { reason, entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::RejectInvite(
                        reason, entity_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyChangeOwner { entity_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::ChangeOwner(
                        entity_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyDelete => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::Delete))
                    .await?;
            }
            ServerMessage::PartyUpdateRules {
                item_sharing,
                xp_sharing,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyUpdateRules {
                        item_sharing,
                        xp_sharing,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberList {
                item_sharing,
                xp_sharing,
                owner_character_id,
                members,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMembers::List {
                        item_sharing,
                        xp_sharing,
                        owner_character_id,
                        members,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberLeave {
                leaver_character_id,
                owner_character_id,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMembers::Leave {
                        leaver_character_id,
                        owner_character_id,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberKicked { character_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::MemberKicked(
                        character_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyMemberDisconnect { character_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyReply::MemberDisconnect(
                        character_id,
                    )))
                    .await?;
            }
            ServerMessage::PartyMemberUpdateInfo { member_info } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberUpdateInfo {
                        member_info,
                    }))
                    .await?;
            }
            ServerMessage::PartyMemberRewardItem {
                client_entity_id,
                item,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerPartyMemberRewardItem {
                        entity_id: client_entity_id,
                        item,
                    }))
                    .await?;
            }
            ServerMessage::ChangeNpcId { entity_id, npc_id } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerChangeNpcId { entity_id, npc_id }))
                    .await?;
            }
            ServerMessage::SetHotbarSlot { slot_index, slot } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerSetHotbarSlot {
                        slot_index,
                        slot,
                    }))
                    .await?;
            }
            ServerMessage::AdjustPosition {
                entity_id,
                position,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerAdjustPosition {
                        entity_id,
                        position,
                    }))
                    .await?;
            }
            ServerMessage::CraftInsertGem { update_items } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCraftItem::InsertGemSuccess {
                        items: update_items,
                    }))
                    .await?;
            }
            ServerMessage::CraftInsertGemError { error } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerCraftItem::InsertGemFailed {
                        error,
                    }))
                    .await?;
            }
            ServerMessage::BankOpen => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerBankOpen::Open))
                    .await?;
            }
            ServerMessage::BankSetItems { items } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerBankOpen::SetItems { items }))
                    .await?;
            }
            ServerMessage::BankUpdateItems { items } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerBankOpen::UpdateItems { items }))
                    .await?;
            }
            ServerMessage::BankTransaction {
                inventory_item_slot,
                inventory_item,
                inventory_money,
                bank_slot,
                bank_item,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerBankTransaction {
                        inventory_item_slot,
                        inventory_item,
                        inventory_money,
                        bank_slot,
                        bank_item,
                    }))
                    .await?;
            }
            ServerMessage::RepairedItemUsingNpc {
                item_slot,
                item,
                updated_money,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerRepairedItemUsingNpc {
                        item_slot,
                        item,
                        updated_money,
                    }))
                    .await?;
            }
            ServerMessage::ClanInfo {
                id,
                mark,
                level,
                points,
                money,
                name,
                description,
                position,
                contribution,
                skills,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerClanCommand::ClanInfo {
                        id,
                        name,
                        description,
                        mark,
                        level,
                        points,
                        money,
                        position,
                        contribution,
                        skills,
                    }))
                    .await?;
            }
            ServerMessage::ClanUpdateInfo {
                id,
                mark,
                level,
                points,
                money,
                skills,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerClanCommand::ClanUpdateInfo {
                        id,
                        mark,
                        level,
                        points,
                        money,
                        skills,
                    }))
                    .await?;
            }
            ServerMessage::CharacterUpdateClan {
                client_entity_id,
                id,
                name,
                mark,
                level,
                position,
            } => {
                client
                    .connection
                    .write_packet(Packet::from(
                        &PacketServerClanCommand::CharacterUpdateClan {
                            client_entity_id,
                            id,
                            name,
                            mark,
                            level,
                            position,
                        },
                    ))
                    .await?;
            }
            ServerMessage::ClanMemberConnected { name, channel_id } => {
                client
                    .connection
                    .write_packet(Packet::from(
                        &PacketServerClanCommand::ClanMemberConnected { name, channel_id },
                    ))
                    .await?;
            }
            ServerMessage::ClanMemberDisconnected { name } => {
                client
                    .connection
                    .write_packet(Packet::from(
                        &PacketServerClanCommand::ClanMemberDisconnected { name },
                    ))
                    .await?;
            }
            ServerMessage::ClanCreateError { error } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerClanCommand::ClanCreateError {
                        error,
                    }))
                    .await?;
            }
            ServerMessage::ClanMemberList { members } => {
                client
                    .connection
                    .write_packet(Packet::from(&PacketServerClanCommand::ClanMemberList {
                        members,
                    }))
                    .await?;
            }
            // These messages are for other servers
            ServerMessage::ReturnToCharacterSelect
            | ServerMessage::LoginSuccess { .. }
            | ServerMessage::LoginError { .. }
            | ServerMessage::ChannelList { .. }
            | ServerMessage::ChannelListError { .. }
            | ServerMessage::JoinServerSuccess { .. }
            | ServerMessage::JoinServerError { .. }
            | ServerMessage::CharacterList { .. }
            | ServerMessage::CharacterListAppend { .. }
            | ServerMessage::CreateCharacterSuccess { .. }
            | ServerMessage::CreateCharacterError { .. }
            | ServerMessage::DeleteCharacterStart { .. }
            | ServerMessage::DeleteCharacterCancel { .. }
            | ServerMessage::DeleteCharacterError { .. }
            | ServerMessage::SelectCharacterSuccess { .. }
            | ServerMessage::SelectCharacterError
            | ServerMessage::UpdateSkillList { .. } => {
                panic!("Received unexpected server message for game server")
            }
        }
        Ok(())
    }
}

implement_protocol_server! { GameServer }
