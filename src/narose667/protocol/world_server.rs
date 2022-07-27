use async_trait::async_trait;

use rose_game_common::messages::server::ServerMessage;
use rose_network_common::{Packet, PacketError};

use crate::{
    implement_protocol_server,
    protocol::{Client, ProtocolServer, ProtocolServerError},
};

pub struct WorldServer;

impl WorldServer {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &mut self,
        _client: &mut Client<'_>,
        packet: &Packet,
    ) -> Result<(), anyhow::Error> {
        dbg!(packet);
        Err(PacketError::InvalidPacket.into())
    }

    async fn handle_server_message(
        &mut self,
        _client: &mut Client<'_>,
        message: ServerMessage,
    ) -> Result<(), anyhow::Error> {
        dbg!(message);
        panic!("Received unexpected server message for world server");
    }
}

implement_protocol_server! { WorldServer }
