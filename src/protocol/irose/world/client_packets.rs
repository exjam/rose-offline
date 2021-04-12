use bytes::{BufMut, BytesMut};
use num_derive::FromPrimitive;
use std::convert::TryFrom;
use std::time::SystemTime;

use crate::protocol::packet::*;
use crate::protocol::ProtocolError;

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
pub struct PacketClientCharacterList {}

impl TryFrom<&Packet> for PacketClientCharacterList {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::CharacterListRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        Ok(PacketClientCharacterList {})
    }
}
