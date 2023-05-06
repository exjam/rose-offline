use async_trait::async_trait;
use thiserror::Error;

use rose_game_common::messages::{client::ClientMessage, server::ServerMessage};
use rose_network_common::{Connection, PacketCodec};

use crate::game::messages::control::ClientType;

pub struct Client<'a> {
    pub entity: bevy::ecs::prelude::Entity,
    pub connection: Connection<'a>,
    pub client_message_tx: crossbeam_channel::Sender<ClientMessage>,
    pub server_message_rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
}

#[derive(Debug, Error)]
pub enum ProtocolServerError {
    #[error("server initiated disconnect")]
    ServerInitiatedDisconnect,
}

#[async_trait]
pub trait ProtocolServer {
    async fn run_client(&mut self, client: &mut Client) -> Result<(), anyhow::Error>;
}

pub struct Protocol {
    pub client_type: ClientType,
    pub packet_codec: Box<dyn PacketCodec + Send + Sync>,
    pub create_server: fn() -> Box<dyn ProtocolServer + Send + Sync>,
}

pub mod server;

#[macro_export]
macro_rules! implement_protocol_server {
    ( $x:ident ) => {
        #[async_trait]
        impl ProtocolServer for $x {
            async fn run_client(&mut self, client: &mut Client) -> Result<(), anyhow::Error> {
                loop {
                    tokio::select! {
                        packet = client.connection.read_packet() => {
                            match packet {
                                Ok(packet) => {
                                    match self.handle_packet(client, &packet) {
                                        Ok(_) => {},
                                        Err(error) => {
                                            log::warn!("RECV [{:03X}] {:02x?}", packet.command, &packet.data[..]);
                                            return Err(error);
                                        },
                                    }
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
    };
}
