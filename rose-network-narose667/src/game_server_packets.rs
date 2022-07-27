use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
}

#[allow(dead_code)]
#[derive(Clone, Copy, FromPrimitive)]
pub enum ConnectResult {
    Ok = 0,
    Failed = 1,
    TimeOut = 2,
    InvalidPassword = 3,
    AlreadyLoggedIn = 4,
}

pub struct PacketConnectionReply {
    pub result: ConnectResult,
    pub packet_sequence_id: u32,
    pub pay_flags: u32,
}

impl TryFrom<&Packet> for PacketConnectionReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::ConnectReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let packet_sequence_id = reader.read_u32()?;
        let pay_flags = reader.read_u32()?;
        Ok(PacketConnectionReply {
            result,
            packet_sequence_id,
            pay_flags,
        })
    }
}

impl From<&PacketConnectionReply> for Packet {
    fn from(packet: &PacketConnectionReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ConnectReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u32(packet.packet_sequence_id);
        writer.write_u32(packet.pay_flags);
        writer.into()
    }
}
