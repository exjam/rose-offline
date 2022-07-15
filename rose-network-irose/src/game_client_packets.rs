use bevy::math::{Vec2, Vec3};
use modular_bitfield::{
    bitfield,
    prelude::{B14, B2},
};
use num_derive::FromPrimitive;
use std::convert::{TryFrom, TryInto};

use rose_data::{AmmoIndex, EquipmentIndex, Item, MotionId, SkillId, VehiclePartIndex, WarpGateId};
use rose_data_irose::{decode_ammo_index, encode_ammo_index};
use rose_game_common::{
    components::{BasicStatType, CharacterUniqueId, HotbarSlot, ItemSlot, SkillSlot},
    messages::{
        client::{NpcStoreBuyItem, ReviveRequestType},
        ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing,
    },
};
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{
    decode_item_slot, encode_item_slot, PacketReadEntityId, PacketReadEquipmentIndex,
    PacketReadHotbarSlot, PacketReadItemSlot, PacketReadItems, PacketReadPartyRules,
    PacketReadSkillSlot, PacketReadVehiclePartIndex, PacketWriteEntityId,
    PacketWriteEquipmentIndex, PacketWriteHotbarSlot, PacketWriteItemSlot, PacketWritePartyRules,
    PacketWriteSkillSlot, PacketWriteVehiclePartIndex,
};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    LogoutRequest = 0x707,
    ConnectRequest = 0x70b,
    ReturnToCharacterSelectRequest = 0x71C,
    QuestRequest = 0x730,
    JoinZone = 0x753,
    ReviveRequest = 0x755,
    SetReviveZone = 0x756,
    MoveCollision = 0x771,
    Emote = 0x781,
    MoveToggle = 0x782,
    Chat = 0x783,
    StopMove = 0x796,
    Attack = 0x798,
    Move = 0x79a,
    NpcStoreTransaction = 0x7a1,
    UseItem = 0x7a3,
    DropItemFromInventory = 0x7a4,
    ChangeEquipment = 0x7a5,
    PickupItemDrop = 0x7a7,
    WarpGateRequest = 0x7a8,
    IncreaseBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
    ChangeAmmo = 0x7ab,
    LevelUpSkill = 0x7b1,
    CastSkillSelf = 0x7b2,
    CastSkillTargetEntity = 0x7b3,
    CastSkillTargetPosition = 0x7b4,
    ChangeVehiclePart = 0x7ca,
    PersonalStoreListItems = 0x7c4,
    PersonalStoreBuyItem = 0x7c5,
    PartyRequest = 0x7d0,
    PartyReply = 0x7d1,
    PartyUpdateRules = 0x7d7,
}

#[derive(Debug)]
pub struct PacketClientConnectRequest<'a> {
    pub login_token: u32,
    pub password_md5: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientConnectRequest<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ConnectRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let login_token = reader.read_u32()?;
        let password_md5 = reader.read_fixed_length_utf8(32)?;
        Ok(PacketClientConnectRequest {
            login_token,
            password_md5,
        })
    }
}

impl<'a> From<&'a PacketClientConnectRequest<'a>> for Packet {
    fn from(packet: &'a PacketClientConnectRequest<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ConnectRequest as u16);
        writer.write_u32(packet.login_token as u32);
        writer.write_fixed_length_utf8(packet.password_md5, 32);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientJoinZone {
    pub weight_rate: u8,
    pub z: u16,
}

impl TryFrom<&Packet> for PacketClientJoinZone {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::JoinZone as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let weight_rate = reader.read_u8()?;
        let z = reader.read_u16()?;
        Ok(PacketClientJoinZone { weight_rate, z })
    }
}

impl From<&PacketClientJoinZone> for Packet {
    fn from(packet: &PacketClientJoinZone) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::JoinZone as u16);
        writer.write_u8(packet.weight_rate);
        writer.write_u16(packet.z);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientMove {
    pub target_entity_id: Option<ClientEntityId>,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl TryFrom<&Packet> for PacketClientMove {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Move as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = reader.read_u16()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_u16()?;
        Ok(PacketClientMove {
            target_entity_id: if target_entity_id == 0 {
                None
            } else {
                Some(ClientEntityId(target_entity_id as usize))
            },
            x,
            y,
            z,
        })
    }
}

impl From<&PacketClientMove> for Packet {
    fn from(packet: &PacketClientMove) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::Move as u16);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientAttack {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientAttack {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Attack as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientAttack { target_entity_id })
    }
}

impl From<&PacketClientAttack> for Packet {
    fn from(packet: &PacketClientAttack) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::Attack as u16);
        writer.write_entity_id(packet.target_entity_id);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientChat<'a> {
    pub text: &'a str,
}

impl<'a> From<&'a PacketClientChat<'a>> for Packet {
    fn from(packet: &'a PacketClientChat<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::Chat as u16);
        writer.write_null_terminated_utf8(packet.text);
        writer.into()
    }
}

impl<'a> TryFrom<&'a Packet> for PacketClientChat<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Chat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketClientChat { text })
    }
}

#[derive(Debug)]
pub struct PacketClientSetHotbarSlot {
    pub slot_index: usize,
    pub slot: Option<HotbarSlot>,
}

impl TryFrom<&Packet> for PacketClientSetHotbarSlot {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SetHotbarSlot as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let slot_index = reader.read_u8()? as usize;
        let slot = reader.read_hotbar_slot()?;
        Ok(PacketClientSetHotbarSlot { slot_index, slot })
    }
}

impl From<&PacketClientSetHotbarSlot> for Packet {
    fn from(packet: &PacketClientSetHotbarSlot) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::SetHotbarSlot as u16);
        writer.write_u8(packet.slot_index as u8);
        writer.write_hotbar_slot(&packet.slot);
        writer.into()
    }
}

pub struct PacketClientChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeEquipment {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeEquipment as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let equipment_index = reader.read_equipment_index_u16()?;
        let item_slot = reader.read_item_slot_u16().ok();
        Ok(PacketClientChangeEquipment {
            equipment_index,
            item_slot,
        })
    }
}

impl From<&PacketClientChangeEquipment> for Packet {
    fn from(packet: &PacketClientChangeEquipment) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ChangeEquipment as u16);
        writer.write_equipment_index_u16(packet.equipment_index);
        if let Some(item_slot) = packet.item_slot {
            writer.write_item_slot_u16(item_slot);
        } else {
            writer.write_u16(0);
        }
        writer.into()
    }
}

pub struct PacketClientChangeVehiclePart {
    pub vehicle_part_index: VehiclePartIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeVehiclePart {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeVehiclePart as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let vehicle_part_index = reader.read_vehicle_part_index_u16()?;
        let item_slot = reader.read_item_slot_u16().ok();
        Ok(PacketClientChangeVehiclePart {
            vehicle_part_index,
            item_slot,
        })
    }
}

impl From<&PacketClientChangeVehiclePart> for Packet {
    fn from(packet: &PacketClientChangeVehiclePart) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ChangeVehiclePart as u16);
        writer.write_vehicle_part_index_u16(packet.vehicle_part_index);
        if let Some(item_slot) = packet.item_slot {
            writer.write_item_slot_u16(item_slot);
        } else {
            writer.write_u16(0);
        }
        writer.into()
    }
}

pub struct PacketClientIncreaseBasicStat {
    pub basic_stat_type: BasicStatType,
}

impl TryFrom<&Packet> for PacketClientIncreaseBasicStat {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::IncreaseBasicStat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let basic_stat_type = match reader.read_u8()? {
            0 => BasicStatType::Strength,
            1 => BasicStatType::Dexterity,
            2 => BasicStatType::Intelligence,
            3 => BasicStatType::Concentration,
            4 => BasicStatType::Charm,
            5 => BasicStatType::Sense,
            _ => return Err(PacketError::InvalidPacket),
        };
        Ok(PacketClientIncreaseBasicStat { basic_stat_type })
    }
}

impl From<&PacketClientIncreaseBasicStat> for Packet {
    fn from(packet: &PacketClientIncreaseBasicStat) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::IncreaseBasicStat as u16);
        match packet.basic_stat_type {
            BasicStatType::Strength => writer.write_u8(0),
            BasicStatType::Dexterity => writer.write_u8(1),
            BasicStatType::Intelligence => writer.write_u8(2),
            BasicStatType::Concentration => writer.write_u8(3),
            BasicStatType::Charm => writer.write_u8(4),
            BasicStatType::Sense => writer.write_u8(5),
        }
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientPickupItemDrop {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPickupItemDrop {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PickupItemDrop as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = reader.read_entity_id()?;
        Ok(PacketClientPickupItemDrop { target_entity_id })
    }
}

impl From<&PacketClientPickupItemDrop> for Packet {
    fn from(packet: &PacketClientPickupItemDrop) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::PickupItemDrop as u16);
        writer.write_entity_id(packet.target_entity_id);
        writer.into()
    }
}

pub struct PacketClientReviveRequest {
    pub revive_request_type: ReviveRequestType,
}

impl TryFrom<&Packet> for PacketClientReviveRequest {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ReviveRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let revive_request_type = match reader.read_u8()? {
            1 => ReviveRequestType::RevivePosition,
            2 => ReviveRequestType::SavePosition,
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(PacketClientReviveRequest {
            revive_request_type,
        })
    }
}

impl From<&PacketClientReviveRequest> for Packet {
    fn from(packet: &PacketClientReviveRequest) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ReviveRequest as u16);

        match packet.revive_request_type {
            ReviveRequestType::RevivePosition => writer.write_u8(1),
            ReviveRequestType::SavePosition => writer.write_u8(2),
        }

        writer.into()
    }
}

pub enum PacketClientQuestRequestType {
    DeleteQuest,
    DoTrigger,
}

pub struct PacketClientQuestRequest {
    pub request_type: PacketClientQuestRequestType,
    pub quest_slot: u8,
    pub quest_id: u32,
}

impl TryFrom<&Packet> for PacketClientQuestRequest {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::QuestRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let request_type = match reader.read_u8()? {
            2 => PacketClientQuestRequestType::DeleteQuest,
            3 => PacketClientQuestRequestType::DoTrigger,
            _ => return Err(PacketError::InvalidPacket),
        };
        let quest_slot = reader.read_u8()?;
        let quest_id = reader.read_u32()?;

        Ok(PacketClientQuestRequest {
            request_type,
            quest_slot,
            quest_id,
        })
    }
}

impl From<&PacketClientQuestRequest> for Packet {
    fn from(packet: &PacketClientQuestRequest) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::QuestRequest as u16);

        let request_type = match packet.request_type {
            PacketClientQuestRequestType::DeleteQuest => 2,
            PacketClientQuestRequestType::DoTrigger => 3,
        };
        writer.write_u8(request_type);
        writer.write_u8(packet.quest_slot);
        writer.write_u32(packet.quest_id);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientPersonalStoreListItems {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreListItems {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreListItems as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = reader.read_entity_id()?;
        Ok(PacketClientPersonalStoreListItems { target_entity_id })
    }
}

impl From<&PacketClientPersonalStoreListItems> for Packet {
    fn from(packet: &PacketClientPersonalStoreListItems) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::PersonalStoreListItems as u16);
        writer.write_entity_id(packet.target_entity_id);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientPersonalStoreBuyItem {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreBuyItem {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreBuyItem as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let store_entity_id = ClientEntityId(reader.read_u16()? as usize);

        // Although the packet supports multiple items, no one uses it
        // so to keep our code simpler we only support single item.
        let _item_count = reader.read_u8()?;

        let store_slot_index = reader.read_u8()? as usize;
        let buy_item = reader.read_item_full()?.ok_or(PacketError::InvalidPacket)?;

        Ok(PacketClientPersonalStoreBuyItem {
            store_entity_id,
            store_slot_index,
            buy_item,
        })
    }
}

#[derive(Debug)]
pub enum PacketClientDropItemFromInventory {
    Item(ItemSlot, u32),
    Money(u32),
}

impl TryFrom<&Packet> for PacketClientDropItemFromInventory {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::DropItemFromInventory as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        // A value of 0 for inventory_index is interpreted to mean dropping money.
        // PacketReader::read_item_slot_u8 returns PacketError for value 0 but in this case it is interpreted to mean dropping money.
        let inventory_index = reader.read_item_slot_u8();
        let quantity = reader.read_u32()?;
        match inventory_index {
            Ok(item_slot) => Ok(PacketClientDropItemFromInventory::Item(item_slot, quantity)),
            Err(_) => Ok(PacketClientDropItemFromInventory::Money(quantity)),
        }
    }
}

impl From<&PacketClientDropItemFromInventory> for Packet {
    fn from(packet: &PacketClientDropItemFromInventory) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::DropItemFromInventory as u16);

        match *packet {
            PacketClientDropItemFromInventory::Item(item_slot, quantity) => {
                writer.write_item_slot_u8(item_slot);
                writer.write_u32(quantity);
            }
            PacketClientDropItemFromInventory::Money(amount) => {
                writer.write_u8(0);
                writer.write_u32(amount);
            }
        }

        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientUseItem {
    pub item_slot: ItemSlot,
    pub target_entity_id: Option<ClientEntityId>,
}

impl TryFrom<&Packet> for PacketClientUseItem {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::UseItem as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_slot = reader.read_item_slot_u16()?;
        let target_entity_id = reader.read_u16().ok().map(|x| ClientEntityId(x as usize));

        Ok(PacketClientUseItem {
            item_slot,
            target_entity_id,
        })
    }
}

impl From<&PacketClientUseItem> for Packet {
    fn from(packet: &PacketClientUseItem) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::UseItem as u16);
        writer.write_item_slot_u16(packet.item_slot);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientLevelUpSkill {
    pub skill_slot: SkillSlot,
    pub next_skill_idx: SkillId,
}

impl TryFrom<&Packet> for PacketClientLevelUpSkill {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::LevelUpSkill as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;
        let next_skill_idx = SkillId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;

        Ok(PacketClientLevelUpSkill {
            skill_slot,
            next_skill_idx,
        })
    }
}

impl From<&PacketClientLevelUpSkill> for Packet {
    fn from(packet: &PacketClientLevelUpSkill) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::LevelUpSkill as u16);
        writer.write_skill_slot_u8(packet.skill_slot);
        writer.write_u16(packet.next_skill_idx.get());
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillSelf {
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillSelf {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillSelf as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillSelf { skill_slot })
    }
}

impl From<&PacketClientCastSkillSelf> for Packet {
    fn from(packet: &PacketClientCastSkillSelf) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::CastSkillSelf as u16);
        writer.write_skill_slot_u8(packet.skill_slot);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetEntity {
    pub target_entity_id: ClientEntityId,
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetEntity as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = reader.read_entity_id()?;
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillTargetEntity {
            target_entity_id,
            skill_slot,
        })
    }
}

impl From<&PacketClientCastSkillTargetEntity> for Packet {
    fn from(packet: &PacketClientCastSkillTargetEntity) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::CastSkillTargetEntity as u16);
        writer.write_entity_id(packet.target_entity_id);
        writer.write_skill_slot_u8(packet.skill_slot);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetPosition {
    pub skill_slot: SkillSlot,
    pub position: Vec2,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetPosition {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetPosition as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;

        Ok(PacketClientCastSkillTargetPosition {
            skill_slot,
            position: Vec2::new(x, y),
        })
    }
}

impl From<&PacketClientCastSkillTargetPosition> for Packet {
    fn from(packet: &PacketClientCastSkillTargetPosition) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::CastSkillTargetPosition as u16);
        writer.write_skill_slot_u8(packet.skill_slot);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientNpcStoreTransactionBuyItem {
    pub tab: u8,
    pub tab_item_index: u8,
    pub quantity: u16,
}

#[derive(Debug)]
pub struct PacketClientNpcStoreTransaction {
    pub npc_entity_id: ClientEntityId,
    pub buy_items: Vec<NpcStoreBuyItem>,
    pub sell_items: Vec<(ItemSlot, usize)>,
}

impl TryFrom<&Packet> for PacketClientNpcStoreTransaction {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::NpcStoreTransaction as u16 {
            return Err(PacketError::InvalidPacket);
        }
        let mut reader = PacketReader::from(packet);

        let npc_entity_id = reader.read_entity_id()?;
        let buy_item_count = reader.read_u8()?;
        let sell_item_count = reader.read_u8()?;
        let _economy_time = reader.read_u32()?;
        let mut buy_items = Vec::new();
        let mut sell_items = Vec::new();

        for _ in 0..buy_item_count {
            let tab_index = reader.read_u8()? as usize;
            let item_index = reader.read_u8()? as usize;
            let quantity = reader.read_u16()? as usize;
            buy_items.push(NpcStoreBuyItem {
                tab_index,
                item_index,
                quantity,
            });
        }

        for _ in 0..sell_item_count {
            let item_slot = reader.read_item_slot_u8()?;
            let quantity = reader.read_u16()? as usize;
            sell_items.push((item_slot, quantity));
        }

        Ok(PacketClientNpcStoreTransaction {
            npc_entity_id,
            buy_items,
            sell_items,
        })
    }
}

impl From<&PacketClientNpcStoreTransaction> for Packet {
    fn from(packet: &PacketClientNpcStoreTransaction) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::NpcStoreTransaction as u16);
        writer.write_entity_id(packet.npc_entity_id);
        writer.write_u8(packet.buy_items.len() as u8);
        writer.write_u8(packet.sell_items.len() as u8);
        writer.write_u32(0); // economy time

        for buy_item in packet.buy_items.iter() {
            writer.write_u8(buy_item.tab_index as u8);
            writer.write_u8(buy_item.item_index as u8);
            writer.write_u16(buy_item.quantity as u16);
        }

        for &(item_slot, quantity) in packet.sell_items.iter() {
            writer.write_item_slot_u8(item_slot);
            writer.write_u16(quantity as u16);
        }

        writer.into()
    }
}

#[bitfield]
#[derive(Clone, Copy)]
struct ChangeAmmoBits {
    ammo_index: B2,
    item_slot: B14,
}

#[derive(Debug)]
pub struct PacketClientChangeAmmo {
    pub ammo_index: AmmoIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeAmmo {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeAmmo as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let bytes = reader.read_fixed_length_bytes(2)?;
        let change_ammo = ChangeAmmoBits::from_bytes(bytes[0..2].try_into().unwrap());
        let item_slot = decode_item_slot(change_ammo.item_slot() as usize);
        let ammo_index = decode_ammo_index(change_ammo.ammo_index() as usize)
            .ok_or(PacketError::InvalidPacket)?;

        Ok(PacketClientChangeAmmo {
            ammo_index,
            item_slot,
        })
    }
}

impl From<&PacketClientChangeAmmo> for Packet {
    fn from(packet: &PacketClientChangeAmmo) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ChangeAmmo as u16);
        let mut ammo_bits = ChangeAmmoBits::new()
            .with_ammo_index(encode_ammo_index(packet.ammo_index).unwrap() as u8);
        if let Some(item_slot) = packet.item_slot {
            ammo_bits = ammo_bits.with_item_slot(encode_item_slot(item_slot) as u16);
        }

        for b in ammo_bits.into_bytes().iter() {
            writer.write_u8(*b);
        }

        writer.into()
    }
}

pub enum PacketClientMoveToggleType {
    Run,
    Sit,
    Drive,
}

pub struct PacketClientMoveToggle {
    pub toggle_type: PacketClientMoveToggleType,
}

impl TryFrom<&Packet> for PacketClientMoveToggle {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::MoveToggle as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let toggle_type = match reader.read_u8()? {
            0 => PacketClientMoveToggleType::Run,
            1 => PacketClientMoveToggleType::Sit,
            2 => PacketClientMoveToggleType::Drive,
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(PacketClientMoveToggle { toggle_type })
    }
}

impl From<&PacketClientMoveToggle> for Packet {
    fn from(packet: &PacketClientMoveToggle) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::MoveToggle as u16);
        match packet.toggle_type {
            PacketClientMoveToggleType::Run => writer.write_u8(0),
            PacketClientMoveToggleType::Sit => writer.write_u8(1),
            PacketClientMoveToggleType::Drive => writer.write_u8(2),
        }
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientEmote {
    pub motion_id: MotionId,
    pub is_stop: bool,
}

impl TryFrom<&Packet> for PacketClientEmote {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Emote as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let motion_id = MotionId::new(reader.read_u16()?);
        let is_stop = reader.read_u16()? != 0;

        Ok(PacketClientEmote { motion_id, is_stop })
    }
}

impl From<&PacketClientEmote> for Packet {
    fn from(packet: &PacketClientEmote) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::Emote as u16);
        writer.write_u16(packet.motion_id.get());
        writer.write_u16(if packet.is_stop { 1 << 15 } else { 0 });
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientWarpGateRequest {
    pub warp_gate_id: WarpGateId,
}

impl TryFrom<&Packet> for PacketClientWarpGateRequest {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::WarpGateRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let warp_gate_id = WarpGateId::new(reader.read_u16()?);
        Ok(PacketClientWarpGateRequest { warp_gate_id })
    }
}

impl From<&PacketClientWarpGateRequest> for Packet {
    fn from(packet: &PacketClientWarpGateRequest) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::WarpGateRequest as u16);
        writer.write_u16(packet.warp_gate_id.get());
        writer.into()
    }
}

#[derive(Debug)]
pub enum PacketClientPartyRequest {
    Create(ClientEntityId),
    Invite(ClientEntityId),
    Leave,
    ChangeOwner(ClientEntityId),
    Kick(CharacterUniqueId),
}

impl TryFrom<&Packet> for PacketClientPartyRequest {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PartyRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let request = match reader.read_u8()? {
            0 => PacketClientPartyRequest::Create(ClientEntityId(reader.read_u16()? as usize)),
            1 => PacketClientPartyRequest::Invite(ClientEntityId(reader.read_u16()? as usize)),
            2 => PacketClientPartyRequest::Leave,
            3 => PacketClientPartyRequest::ChangeOwner(ClientEntityId(reader.read_u16()? as usize)),
            0x81 => PacketClientPartyRequest::Kick(reader.read_u32()?),
            _ => return Err(PacketError::InvalidPacket),
        };
        Ok(request)
    }
}

impl From<&PacketClientPartyRequest> for Packet {
    fn from(packet: &PacketClientPartyRequest) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::PartyRequest as u16);
        match *packet {
            PacketClientPartyRequest::Create(entity_id) => {
                writer.write_u8(0);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketClientPartyRequest::Invite(entity_id) => {
                writer.write_u8(1);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketClientPartyRequest::Leave => {
                writer.write_u8(2);
                writer.write_u32(0);
            }
            PacketClientPartyRequest::ChangeOwner(entity_id) => {
                writer.write_u8(3);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketClientPartyRequest::Kick(unique_id) => {
                writer.write_u8(0x81);
                writer.write_u32(unique_id);
            }
        }
        writer.into()
    }
}

#[derive(Debug)]
pub enum PacketClientPartyReply {
    AcceptCreate(ClientEntityId),
    AcceptJoin(ClientEntityId),
    Reject(PartyRejectInviteReason, ClientEntityId),
}

impl TryFrom<&Packet> for PacketClientPartyReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PartyReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let reply = match reader.read_u8()? {
            1 => PacketClientPartyReply::Reject(
                PartyRejectInviteReason::Busy,
                ClientEntityId(reader.read_u16()? as usize),
            ),
            2 => PacketClientPartyReply::AcceptCreate(ClientEntityId(reader.read_u16()? as usize)),
            3 => PacketClientPartyReply::AcceptJoin(ClientEntityId(reader.read_u16()? as usize)),
            4 => PacketClientPartyReply::Reject(
                PartyRejectInviteReason::Reject,
                ClientEntityId(reader.read_u16()? as usize),
            ),
            _ => return Err(PacketError::InvalidPacket),
        };
        Ok(reply)
    }
}

impl From<&PacketClientPartyReply> for Packet {
    fn from(packet: &PacketClientPartyReply) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::PartyReply as u16);
        match *packet {
            PacketClientPartyReply::AcceptCreate(entity_id) => {
                writer.write_u8(2);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketClientPartyReply::AcceptJoin(entity_id) => {
                writer.write_u8(3);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketClientPartyReply::Reject(reason, entity_id) => {
                match reason {
                    PartyRejectInviteReason::Busy => writer.write_u8(1),
                    PartyRejectInviteReason::Reject => writer.write_u8(4),
                }
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
        }
        writer.into()
    }
}

pub struct PacketClientPartyUpdateRules {
    pub item_sharing: PartyItemSharing,
    pub xp_sharing: PartyXpSharing,
}

impl TryFrom<&Packet> for PacketClientPartyUpdateRules {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PartyUpdateRules as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let (item_sharing, xp_sharing) = reader.read_party_rules()?;
        Ok(PacketClientPartyUpdateRules {
            item_sharing,
            xp_sharing,
        })
    }
}

impl From<&PacketClientPartyUpdateRules> for Packet {
    fn from(packet: &PacketClientPartyUpdateRules) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::PartyUpdateRules as u16);
        writer.write_party_rules(&packet.item_sharing, &packet.xp_sharing);
        writer.into()
    }
}

pub struct PacketClientMoveCollision {
    pub position: Vec3,
}

impl TryFrom<&Packet> for PacketClientMoveCollision {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::MoveCollision as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_i16()? as f32;
        Ok(PacketClientMoveCollision {
            position: Vec3::new(x, y, z),
        })
    }
}

impl From<&PacketClientMoveCollision> for Packet {
    fn from(packet: &PacketClientMoveCollision) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::MoveCollision as u16);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        writer.write_i16(packet.position.z as i16);
        writer.into()
    }
}
