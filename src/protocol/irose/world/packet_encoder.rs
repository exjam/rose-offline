use bytes::BytesMut;

use crate::game::messages::server::*;
use crate::protocol::packet::{Packet, PacketEncoder, PacketWriter};

pub struct WorldPacketEncoder {}

impl WorldPacketEncoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PacketEncoder for WorldPacketEncoder {
    fn encode(self: &Self, message: &ServerMessage) -> Packet {
        Packet::with_data(123, BytesMut::new())
    }
}
