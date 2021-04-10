use crate::game::messages::client::*;
use crate::protocol::{
    packet::{Packet, PacketDecoder, PacketReader},
    ProtocolError,
};

pub struct GamePacketDecoder {}

impl GamePacketDecoder {
    pub fn new() -> Self {
        Self {}
    }
}

impl PacketDecoder for GamePacketDecoder {
    fn decode(self: &Self, packet: &Packet) -> Result<ClientMessage, ProtocolError> {
        Err(ProtocolError::InvalidPacket)
    }
}
