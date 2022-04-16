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
