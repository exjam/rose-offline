use crate::game::messages::client::*;
use crate::protocol::{
    packet::{Packet, PacketDecoder, PacketReader},
    ProtocolError,
};

pub struct WorldPacketDecoder {}

impl WorldPacketDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PacketDecoder for WorldPacketDecoder {
    fn decode(self: &Self, packet: &Packet) -> Result<ClientMessage, ProtocolError> {
        Err(ProtocolError::InvalidPacket)
    }
}
