use std::convert::TryFrom;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    game::components::{
        BasicStatType, ClientEntityId, EquipmentIndex, HotbarSlot, InventoryPageType, ItemSlot,
        INVENTORY_PAGE_SIZE,
    },
    protocol::{Packet, PacketReader, ProtocolError},
};

use super::common_packets::read_hotbar_slot;

#[derive(FromPrimitive)]
pub enum ClientPackets {
    ConnectRequest = 0x70b,
    JoinZone = 0x753,
    Chat = 0x783,
    StopMove = 0x796,
    Attack = 0x798,
    Move = 0x79a,
    DropItem = 0x7a4,
    ChangeEquipment = 0x7a5,
    PickupDroppedItem = 0x7a7,
    IncreaseBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
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
        let slot = read_hotbar_slot(&mut reader)?;
        Ok(PacketClientSetHotbarSlot { slot_index, slot })
    }
}

fn item_slot_from_index(index: usize) -> Option<ItemSlot> {
    if index == 0 {
        None // Invalid
    } else if index < 12 {
        Some(ItemSlot::Equipped(FromPrimitive::from_usize(index)?))
    } else {
        let index = index - 12;
        let page = index / INVENTORY_PAGE_SIZE;
        let slot = index % INVENTORY_PAGE_SIZE;
        match page {
            0 => Some(ItemSlot::Inventory(InventoryPageType::Equipment, slot)),
            1 => Some(ItemSlot::Inventory(InventoryPageType::Consumables, slot)),
            2 => Some(ItemSlot::Inventory(InventoryPageType::Materials, slot)),
            3 => Some(ItemSlot::Inventory(InventoryPageType::Vehicles, slot)),
            _ => None,
        }
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
        let inventory_index = reader.read_u16()? as usize;
        Ok(PacketClientChangeEquipment {
            equipment_index,
            item_slot: item_slot_from_index(inventory_index),
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
