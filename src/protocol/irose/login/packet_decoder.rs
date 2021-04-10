use crate::game::messages::client::*;
use crate::protocol::{
    packet::{Packet, PacketDecoder, PacketReader},
    ProtocolError,
};

pub struct LoginPacketDecoder {}

impl LoginPacketDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PacketDecoder for LoginPacketDecoder {
    fn decode(self: &Self, packet: &Packet) -> Result<ClientMessage, ProtocolError> {
        Err(ProtocolError::InvalidPacket)
    }
}
