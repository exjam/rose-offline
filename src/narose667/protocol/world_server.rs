use async_trait::async_trait;

use rose_game_common::messages::server::ServerMessage;
use rose_network_common::{Packet, PacketError};

use crate::protocol::{Client, ProtocolServer, ProtocolServerError};

pub struct WorldServer;

impl WorldServer {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet(
        &mut self,
        _client: &mut Client<'_>,
        packet: Packet,
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

#[async_trait]
impl ProtocolServer for WorldServer {
    async fn run_client(&mut self, client: &mut Client) -> Result<(), anyhow::Error> {
        loop {
            tokio::select! {
                packet = client.connection.read_packet() => {
                    match packet {
                        Ok(packet) => {
                            self.handle_packet(client, packet).await?;
                        },
                        Err(error) => {
                            return Err(error);
                        }
                    }
                },
                server_message = client.server_message_rx.recv() => {
                    if let Some(message) = server_message {
                        self.handle_server_message(client, message).await?;
                    } else {
                        return Err(ProtocolServerError::ServerInitiatedDisconnect.into());
                    }
                }
            };
        }
    }
}
