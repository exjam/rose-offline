use bytes::{BufMut, BytesMut};
use num_derive::FromPrimitive;
use std::convert::TryFrom;
use std::time::SystemTime;

use crate::protocol::packet::*;
use crate::protocol::ProtocolError;

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
    CharacterListReply = 0x712,
    CreateCharacterReply = 0x713,
    DeleteCharacterReply = 0x714,
    MoveServer = 0x711,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
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

impl From<&PacketConnectionReply> for Packet {
    fn from(packet: &PacketConnectionReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ConnectReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u32(packet.packet_sequence_id);
        writer.write_u32(packet.pay_flags);
        writer.into()
    }
}
