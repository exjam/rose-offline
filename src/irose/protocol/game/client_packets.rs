use std::convert::TryFrom;

use nalgebra::Point2;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    data::item::Item,
    game::{
        components::{
            BasicStatType, ClientEntityId, EquipmentIndex, HotbarSlot, ItemSlot, SkillSlot,
        },
        messages::client::ReviveRequestType,
    },
    irose::protocol::game::common_packets::{
        PacketReadHotbarSlot, PacketReadItemSlot, PacketReadItems, PacketReadSkillSlot,
    },
    protocol::{Packet, PacketReader, ProtocolError},
};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    LogoutRequest = 0x707,
    ConnectRequest = 0x70b,
    ReturnToCharacterSelectRequest = 0x71C,
    QuestRequest = 0x730,
    JoinZone = 0x753,
    ReviveRequest = 0x755,
    Chat = 0x783,
    StopMove = 0x796,
    Attack = 0x798,
    Move = 0x79a,
    UseItem = 0x7a3,
    DropItem = 0x7a4,
    ChangeEquipment = 0x7a5,
    PickupDroppedItem = 0x7a7,
    IncreaseBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
    CastSkillSelf = 0x7b2,
    CastSkillTargetEntity = 0x7b3,
    CastSkillTargetPosition = 0x7b4,
    PersonalStoreListItems = 0x7c4,
    PersonalStoreBuyItem = 0x7c5,
    MoveToggle = 0x782,
}

#[derive(Debug)]
pub struct PacketClientConnectRequest<'a> {
    pub login_token: u32,
    pub password_md5: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientConnectRequest<'a> {
    type Error = ProtocolError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ConnectRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
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

#[derive(Debug)]
pub struct PacketClientJoinZone {
    pub weight_rate: u8,
    pub z: u16,
}

impl TryFrom<&Packet> for PacketClientJoinZone {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::JoinZone as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let weight_rate = reader.read_u8()?;
        let z = reader.read_u16()?;
        Ok(PacketClientJoinZone { weight_rate, z })
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
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Move as u16 {
            return Err(ProtocolError::InvalidPacket);
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

#[derive(Debug)]
pub struct PacketClientAttack {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientAttack {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Attack as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientAttack { target_entity_id })
    }
}

#[derive(Debug)]
pub struct PacketClientChat<'a> {
    pub text: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientChat<'a> {
    type Error = ProtocolError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::Chat as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketClientChat { text })
    }
}

#[derive(Debug)]
pub struct PacketClientSetHotbarSlot {
    pub slot_index: u8,
    pub slot: Option<HotbarSlot>,
}

impl TryFrom<&Packet> for PacketClientSetHotbarSlot {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SetHotbarSlot as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let slot_index = reader.read_u8()?;
        let slot = reader.read_hotbar_slot()?;
        Ok(PacketClientSetHotbarSlot { slot_index, slot })
    }
}

pub struct PacketClientChangeEquipment {
    pub equipment_index: EquipmentIndex,
    pub item_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketClientChangeEquipment {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChangeEquipment as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let equipment_index =
            FromPrimitive::from_u16(reader.read_u16()?).ok_or(ProtocolError::InvalidPacket)?;
        let item_slot = reader.read_item_slot_u16().ok();
        Ok(PacketClientChangeEquipment {
            equipment_index,
            item_slot,
        })
    }
}

pub struct PacketClientIncreaseBasicStat {
    pub basic_stat_type: BasicStatType,
}

impl TryFrom<&Packet> for PacketClientIncreaseBasicStat {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::IncreaseBasicStat as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let basic_stat_type = match reader.read_u8()? {
            0 => BasicStatType::Strength,
            1 => BasicStatType::Dexterity,
            2 => BasicStatType::Intelligence,
            3 => BasicStatType::Concentration,
            4 => BasicStatType::Charm,
            5 => BasicStatType::Sense,
            _ => return Err(ProtocolError::InvalidPacket),
        };
        Ok(PacketClientIncreaseBasicStat { basic_stat_type })
    }
}

#[derive(Debug)]
pub struct PacketClientPickupDroppedItem {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPickupDroppedItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PickupDroppedItem as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientPickupDroppedItem { target_entity_id })
    }
}

pub struct PacketClientReviveRequest {
    pub revive_request_type: ReviveRequestType,
}

impl TryFrom<&Packet> for PacketClientReviveRequest {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ReviveRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let revive_request_type = match reader.read_u8()? {
            1 => ReviveRequestType::RevivePosition,
            2 => ReviveRequestType::SavePosition,
            _ => return Err(ProtocolError::InvalidPacket),
        };

        Ok(PacketClientReviveRequest {
            revive_request_type,
        })
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
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::QuestRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let request_type = match reader.read_u8()? {
            2 => PacketClientQuestRequestType::DeleteQuest,
            3 => PacketClientQuestRequestType::DoTrigger,
            _ => return Err(ProtocolError::InvalidPacket),
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

#[derive(Debug)]
pub struct PacketClientPersonalStoreListItems {
    pub target_entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreListItems {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreListItems as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        Ok(PacketClientPersonalStoreListItems { target_entity_id })
    }
}

#[derive(Debug)]
pub struct PacketClientPersonalStoreBuyItem {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub buy_item: Item,
}

impl TryFrom<&Packet> for PacketClientPersonalStoreBuyItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::PersonalStoreBuyItem as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let store_entity_id = ClientEntityId(reader.read_u16()? as usize);

        // Although the packet supports multiple items, no one uses it
        // so to keep our code simpler we only support single item.
        let _item_count = reader.read_u8()?;

        let store_slot_index = reader.read_u8()? as usize;
        let buy_item = reader
            .read_item_full()?
            .ok_or(ProtocolError::InvalidPacket)?;

        Ok(PacketClientPersonalStoreBuyItem {
            store_entity_id,
            store_slot_index,
            buy_item,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientDropItem {
    pub item_slot: ItemSlot,
    pub quantity: u32,
}

impl TryFrom<&Packet> for PacketClientDropItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::DropItem as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_slot = reader.read_item_slot_u8()?;
        let quantity = reader.read_u32()?;

        Ok(PacketClientDropItem {
            item_slot,
            quantity,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientUseItem {
    pub item_slot: ItemSlot,
    pub target_entity_id: Option<ClientEntityId>,
}

impl TryFrom<&Packet> for PacketClientUseItem {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::UseItem as u16 {
            return Err(ProtocolError::InvalidPacket);
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

#[derive(Debug)]
pub struct PacketClientCastSkillSelf {
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillSelf {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillSelf as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillSelf { skill_slot })
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetEntity {
    pub target_entity_id: ClientEntityId,
    pub skill_slot: SkillSlot,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetEntity {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetEntity as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let target_entity_id = ClientEntityId(reader.read_u16()? as usize);
        let skill_slot = reader.read_skill_slot_u8()?;

        Ok(PacketClientCastSkillTargetEntity {
            target_entity_id,
            skill_slot,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientCastSkillTargetPosition {
    pub skill_slot: SkillSlot,
    pub position: Point2<f32>,
}

impl TryFrom<&Packet> for PacketClientCastSkillTargetPosition {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CastSkillTargetPosition as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;

        Ok(PacketClientCastSkillTargetPosition {
            skill_slot,
            position: Point2::new(x, y),
        })
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
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::MoveToggle as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let toggle_type = match reader.read_u8()? {
            0 => PacketClientMoveToggleType::Run,
            1 => PacketClientMoveToggleType::Sit,
            2 => PacketClientMoveToggleType::Drive,
            _ => return Err(ProtocolError::InvalidPacket),
        };

        Ok(PacketClientMoveToggle { toggle_type })
    }
}
