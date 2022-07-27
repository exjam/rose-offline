use num_derive::FromPrimitive;
use std::convert::TryFrom;

use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    Connect = 0x703,
    ChannelList = 0x704,
    LoginRequest = 0x708,
    SelectServer = 0x70a,
}

pub struct PacketClientConnect;

impl From<&PacketClientConnect> for Packet {
    fn from(_: &PacketClientConnect) -> Self {
        let writer = PacketWriter::new(ClientPackets::Connect as u16);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientLoginRequest<'a> {
    pub username: &'a str,
    pub password_md5: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketClientLoginRequest<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::LoginRequest as u16 {
            return Err(PacketError::InvalidPacket);
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

impl<'a> From<&'a PacketClientLoginRequest<'a>> for Packet {
    fn from(packet: &'a PacketClientLoginRequest<'a>) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::LoginRequest as u16);
        writer.write_fixed_length_utf8(packet.password_md5, 32);
        writer.write_null_terminated_utf8(packet.username);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientChannelList {
    pub server_id: usize,
}

impl TryFrom<&Packet> for PacketClientChannelList {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::ChannelList as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let server_id = reader.read_u32()? as usize;

        Ok(PacketClientChannelList { server_id })
    }
}

impl From<&PacketClientChannelList> for Packet {
    fn from(packet: &PacketClientChannelList) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::ChannelList as u16);
        writer.write_u32(packet.server_id as u32);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketClientSelectServer {
    pub server_id: usize,
    pub channel_id: usize,
}

impl TryFrom<&Packet> for PacketClientSelectServer {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ClientPackets::SelectServer as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let server_id = reader.read_u32()? as usize;
        let channel_id = (reader.read_u8()? - 1) as usize;

        Ok(PacketClientSelectServer {
            server_id,
            channel_id,
        })
    }
}

impl From<&PacketClientSelectServer> for Packet {
    fn from(packet: &PacketClientSelectServer) -> Self {
        let mut writer = PacketWriter::new(ClientPackets::SelectServer as u16);
        writer.write_u32(packet.server_id as u32);
        writer.write_u8((packet.channel_id + 1) as u8);
        writer.into()
    }
}
