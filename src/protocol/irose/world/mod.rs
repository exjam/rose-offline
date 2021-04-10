use crate::game::messages::server::ServerMessage;
use crate::protocol::{packet::Packet, Client, ProtocolClient, ProtocolError};
use async_trait::async_trait;

mod client_packets;
mod server_packets;

pub struct WorldClient {}

impl WorldClient {
    pub fn new() -> Self {
        Self {}
    }

    async fn handle_packet<'a>(
        &self,
        client: &mut Client<'a>,
        packet: Packet,
    ) -> Result<(), ProtocolError> {
        Err(ProtocolError::InvalidPacket)
    }

    async fn handle_server_message<'a>(
        &self,
        client: &mut Client<'a>,
        message: ServerMessage,
    ) -> Result<(), ProtocolError> {
        match message {
            _ => {
                panic!("Unimplemented message for irose world server!")
            }
        }
    }
}

#[async_trait]
impl ProtocolClient for WorldClient {
    async fn run_client(&self, client: &mut Client) -> Result<(), ProtocolError> {
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
                Some(message) = client.server_message_rx.recv() => {
                    self.handle_server_message(client, message).await?;
                }
            };
        }
    }
}
