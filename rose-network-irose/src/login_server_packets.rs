use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ChannelList = 0x704,
    LoginReply = 0x708,
    SelectServer = 0x70a,
    NetworkStatus = 0x7ff,
}

#[allow(dead_code)]
#[derive(Copy, Clone, FromPrimitive)]
pub enum ConnectionResult {
    Connect = 1,
    Accepted = 2,
    Disconnect = 3,
    ServerDead = 4,
}

pub struct PacketConnectionReply {
    pub status: ConnectionResult,
    pub packet_sequence_id: u32,
}

impl TryFrom<&Packet> for PacketConnectionReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::NetworkStatus as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let status = reader.read_u8()?;
        let packet_sequence_id = reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;
        reader.read_u32()?;

        Ok(PacketConnectionReply {
            status: FromPrimitive::from_u8(status).ok_or(PacketError::InvalidPacket)?,
            packet_sequence_id,
        })
    }
}

impl From<&PacketConnectionReply> for Packet {
    fn from(packet: &PacketConnectionReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::NetworkStatus as u16);
        writer.write_u8(packet.status as u8);
        writer.write_u32(packet.packet_sequence_id);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.into()
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum LoginResult {
    Ok = 0,
    Failed = 1,
    UnknownAccount = 2,
    InvalidPassword = 3,
    AlreadyLoggedIn = 4,
    RefusedAccount = 5,
    NeedCharge = 6,
    NoRightToConnect = 7,
    TooManyUser = 8,
    NoRealName = 9,
    InvalidVersion = 10,
    OutsideRegion = 11,
}

pub struct PacketServerLoginReply {
    pub result: LoginResult,
    pub rights: u16,
    pub pay_type: u16,
    pub servers: Vec<(u32, String)>,
}

impl PacketServerLoginReply {
    pub fn with_error_result(result: LoginResult) -> Self {
        PacketServerLoginReply {
            result,
            rights: 0,
            pay_type: 0,
            servers: Vec::new(),
        }
    }
}

impl TryFrom<&Packet> for PacketServerLoginReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::LoginReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let rights = reader.read_u16()?;
        let pay_type = reader.read_u16()?;
        let mut servers = Vec::new();
        while let Ok(name) = reader.read_null_terminated_utf8() {
            let id = reader.read_u32()?;
            servers.push((id, name.into()));
        }

        Ok(PacketServerLoginReply {
            result,
            rights,
            pay_type,
            servers,
        })
    }
}

impl From<&PacketServerLoginReply> for Packet {
    fn from(packet: &PacketServerLoginReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LoginReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u16(packet.rights);
        writer.write_u16(packet.pay_type);

        if packet.result == LoginResult::Ok {
            for (id, name) in packet.servers.iter() {
                writer.write_null_terminated_utf8(name);
                writer.write_u32(*id);
            }
        }

        writer.into()
    }
}

pub struct PacketServerChannelListItem<'a> {
    pub id: u8,
    pub low_age: u8,
    pub high_age: u8,
    pub percent_full: u16,
    pub name: &'a str,
}

pub struct PacketServerChannelList<'a> {
    pub server_id: usize,
    pub channels: Vec<PacketServerChannelListItem<'a>>,
}

impl<'a> TryFrom<&'a Packet> for PacketServerChannelList<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::ChannelList as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let server_id = reader.read_u32()? as usize;

        let mut channels = Vec::new();
        while let Ok(id) = reader.read_u8() {
            let low_age = reader.read_u8()?;
            let high_age = reader.read_u8()?;
            let percent_full = reader.read_u16()?;
            let name = reader.read_null_terminated_utf8()?;
            channels.push(PacketServerChannelListItem {
                id: id - 1,
                low_age,
                high_age,
                percent_full,
                name,
            });
        }

        Ok(PacketServerChannelList {
            server_id,
            channels,
        })
    }
}

impl<'a> From<&PacketServerChannelList<'a>> for Packet {
    fn from(packet: &PacketServerChannelList) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ChannelList as u16);
        writer.write_u32(packet.server_id as u32);
        writer.write_u8(packet.channels.len() as u8);

        for channel in packet.channels.iter() {
            writer.write_u8(channel.id + 1);
            writer.write_u8(channel.low_age);
            writer.write_u8(channel.high_age);
            writer.write_u16(channel.percent_full);
            writer.write_null_terminated_utf8(channel.name);
        }

        writer.into()
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, FromPrimitive)]
pub enum SelectServerResult {
    Ok = 0,
    Failed = 1,
    Full = 2,
    InvalidChannel = 3,
    InactiveChannel = 4,
    InvalidAge = 5,
    NeedCharge = 6,
}

pub struct PacketServerSelectServer<'a> {
    pub result: SelectServerResult,
    pub login_token: u32,
    pub packet_codec_seed: u32,
    pub ip: &'a str,
    pub port: u16,
}

impl PacketServerSelectServer<'_> {
    pub fn with_result(result: SelectServerResult) -> PacketServerSelectServer<'static> {
        PacketServerSelectServer {
            result,
            login_token: 0u32,
            packet_codec_seed: 0u32,
            ip: "",
            port: 0,
        }
    }
}

impl<'a> TryFrom<&'a Packet> for PacketServerSelectServer<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SelectServer as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let login_token = reader.read_u32()?;
        let packet_codec_seed = reader.read_u32()?;
        let ip = reader.read_null_terminated_utf8()?;
        let port = reader.read_u16()?;

        Ok(PacketServerSelectServer {
            result,
            login_token,
            packet_codec_seed,
            ip,
            port,
        })
    }
}

impl<'a> From<&PacketServerSelectServer<'a>> for Packet {
    fn from(packet: &PacketServerSelectServer) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SelectServer as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u32(packet.login_token);
        writer.write_u32(packet.packet_codec_seed);
        writer.write_null_terminated_utf8(packet.ip);
        writer.write_u16(packet.port);
        writer.into()
    }
}
