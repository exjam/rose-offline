use num_derive::FromPrimitive;
use std::convert::TryFrom;

use rose_game_common::components::CharacterGender;
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{PacketReadCharacterGender, PacketWriteCharacterGender};

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

impl<'a> From<&'a PacketClientConnectRequest<'a>> for Packet {
    fn from(packet: &'a PacketClientConnectRequest<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ConnectRequest as u16);
        writer.write_u32(packet.login_token as u32);
        writer.write_fixed_length_utf8(packet.password_md5, 32);
        writer.into()
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

impl From<&PacketClientCharacterList> for Packet {
    fn from(_: &PacketClientCharacterList) -> Self {
        PacketWriter::new(ClientPackets::CharacterListRequest as u16).into()
    }
}

#[derive(Debug)]
pub struct PacketClientCreateCharacter<'a> {
    pub gender: CharacterGender,
    pub hair_color: i32,
    pub hair_style: i32,
    pub face: i32,
    pub weapon_type: i32,
    pub start_point: i32,
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
        let hair_color = reader.read_i32()?;
        let hair_style = reader.read_i32()?;
        let face = reader.read_i32()?;
        let weapon_type = reader.read_i32()?;
        let start_point = reader.read_i32()?;
        let name = reader.read_null_terminated_utf8()?;
        Ok(PacketClientCreateCharacter {
            gender,
            hair_color,
            hair_style,
            face,
            weapon_type,
            start_point,
            name,
        })
    }
}

impl<'a> From<&'a PacketClientCreateCharacter<'a>> for Packet {
    fn from(packet: &'a PacketClientCreateCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::CreateCharacter as u16);
        writer.write_character_gender_u8(packet.gender);
        writer.write_i32(packet.hair_color);
        writer.write_i32(packet.hair_style);
        writer.write_i32(packet.face);
        writer.write_i32(packet.weapon_type);
        writer.write_i32(packet.start_point);
        writer.write_null_terminated_utf8(packet.name);
        writer.into()
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

impl<'a> From<&'a PacketClientDeleteCharacter<'a>> for Packet {
    fn from(packet: &'a PacketClientDeleteCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::DeleteCharacter as u16);
        writer.write_u8(packet.slot);
        writer.write_u8(if packet.is_delete { 1 } else { 0 });
        writer.write_null_terminated_utf8(packet.name);
        writer.into()
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

impl<'a> From<&'a PacketClientSelectCharacter<'a>> for Packet {
    fn from(packet: &'a PacketClientSelectCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::SelectCharacter as u16);
        writer.write_u8(packet.slot);
        writer.write_u8(0);
        writer.write_u8(0);
        writer.write_null_terminated_utf8(packet.name);
        writer.into()
    }
}
