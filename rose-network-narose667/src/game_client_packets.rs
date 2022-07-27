use num_derive::FromPrimitive;
use std::convert::TryFrom;

use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

#[derive(FromPrimitive)]
pub enum ClientPackets {
    ConnectRequest = 0x70b,
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
