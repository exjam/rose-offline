use num_derive::FromPrimitive;
use std::convert::TryFrom;

use crate::protocol::{Packet, PacketReader, ProtocolError};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    Connect = 0x703,
    ChannelList = 0x704,
    LoginRequest = 0x708,
    SelectServer = 0x70a,
}

#[derive(Debug)]
pub struct PacketClientLoginRequest<'a> {
    pub username: &'a str,
    pub password_md5: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientLoginRequest<'a> {
    type Error = ProtocolError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::LoginRequest as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let password_md5 = reader.read_fixed_length_utf8(32)?;
        let username = reader.read_null_terminated_utf8()?;

        Ok(PacketClientLoginRequest {
            username,
            password_md5,
        })
    }
}

#[derive(Debug)]
pub struct PacketClientChannelList {
    pub server_id: u32,
}

impl TryFrom<&Packet> for PacketClientChannelList {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChannelList as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let server_id = reader.read_u32()?;

        Ok(PacketClientChannelList { server_id })
    }
}

#[derive(Debug)]
pub struct PacketClientSelectServer {
    pub server_id: u32,
    pub channel_id: u8,
}

impl TryFrom<&Packet> for PacketClientSelectServer {
    type Error = ProtocolError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SelectServer as u16 {
            return Err(ProtocolError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let server_id = reader.read_u32()?;
        let channel_id = reader.read_u8()? - 1;

        Ok(PacketClientSelectServer {
            server_id,
            channel_id,
        })
    }
}
