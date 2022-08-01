use num_derive::FromPrimitive;
use rose_game_common::messages::ClientEntityId;
use std::convert::TryFrom;

use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{PacketReadEntityId, PacketWriteEntityId};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    ConnectRequest = 0x70b,
    SelectCharacter = 0x715,
    JoinZone = 0x753,
    Chat = 0x783,
    Move = 0x79a,
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
        let target_entity_id = reader.read_option_entity_id()?;
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
