use std::convert::TryFrom;

use num_derive::FromPrimitive;

use crate::{
    game::components::HotbarSlot,
    protocol::{
        packet::{Packet, PacketReader},
        ProtocolError,
    },
};

use super::common_packets::read_hotbar_slot;

#[derive(FromPrimitive)]
pub enum ClientPackets {
    ConnectRequest = 0x70b,
    JoinZone = 0x753,
    Chat = 0x783,
    StopMove = 0x796,
    Move = 0x79a,
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
    pub target_entity_id: u16,
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
            target_entity_id,
            x,
            y,
            z,
        })
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
