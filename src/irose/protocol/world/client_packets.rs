use num_derive::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::components::CharacterGender;
use rose_network_common::{Packet, PacketError, PacketReader};

use crate::irose::protocol::common_packets::PacketReadCharacterGender;

#[derive(FromPrimitive)]
pub enum ClientPackets {
    ConnectRequest = 0x70b,
    CharacterListRequest = 0x712,
    CreateCharacter = 0x713,
    DeleteCharacter = 0x714,
    SelectCharacter = 0x715,
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

#[derive(Debug)]
pub struct PacketClientCharacterList {}

impl TryFrom<&Packet> for PacketClientCharacterList {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CharacterListRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        Ok(PacketClientCharacterList {})
    }
}

#[derive(Debug)]
pub struct PacketClientCreateCharacter<'a> {
    pub gender: CharacterGender,
    pub birth_stone: u8,
    pub hair: u8,
    pub face: u8,
    pub start_point: u16,
    pub name: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientCreateCharacter<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CreateCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }
        let mut reader = PacketReader::from(packet);
        let gender = reader.read_character_gender_u8()?;
        let birth_stone = reader.read_u8()?;
        let hair = reader.read_u8()?;
        let face = reader.read_u8()?;
        let _weapon_type = reader.read_u8()?;
        let start_point = reader.read_u16()?;
        let name = reader.read_null_terminated_utf8()?;
        Ok(PacketClientCreateCharacter {
            gender,
            birth_stone,
            hair,
            face,
            start_point,
            name,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientDeleteCharacter<'a> {
    pub slot: u8,
    pub is_delete: bool,
    pub name: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientDeleteCharacter<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::DeleteCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }
        let mut reader = PacketReader::from(packet);
        let slot = reader.read_u8()?;
        let is_delete = reader.read_u8()? != 0;
        let name = reader.read_null_terminated_utf8()?;
        Ok(PacketClientDeleteCharacter {
            slot,
            is_delete,
            name,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientSelectCharacter<'a> {
    pub slot: u8,
    pub name: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientSelectCharacter<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SelectCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }
        let mut reader = PacketReader::from(packet);
        let slot = reader.read_u8()?;
        let _run_mode = reader.read_u8()?;
        let _ride_mode = reader.read_u8()?;
        let name = reader.read_null_terminated_utf8()?;
        Ok(PacketClientSelectCharacter { slot, name })
    }
}
