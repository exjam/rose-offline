#[derive(Debug)]
pub enum ProtocolError {
    Disconnect,
    IscError,
    InvalidPacket,
    ServerInitiatedDisconnect,
}

impl From<tokio::sync::oneshot::error::RecvError> for ProtocolError {
    fn from(_: tokio::sync::oneshot::error::RecvError) -> ProtocolError {
        ProtocolError::IscError
    }
}

impl From<crossbeam_channel::SendError<crate::game::messages::client::ClientMessage>>
    for ProtocolError
{
    fn from(
        _: crossbeam_channel::SendError<crate::game::messages::client::ClientMessage>,
    ) -> ProtocolError {
        ProtocolError::IscError
    }
}

impl From<crossbeam_channel::SendError<crate::game::messages::control::ControlMessage>>
    for ProtocolError
{
    fn from(
        _: crossbeam_channel::SendError<crate::game::messages::control::ControlMessage>,
    ) -> ProtocolError {
        ProtocolError::IscError
    }
}

mod packet;
pub use packet::Packet;
pub use packet::PacketCodec;
pub use packet::PacketReader;
pub use packet::PacketWriter;

mod connection;
use crate::game::messages::client::ClientMessage;
use crate::game::messages::control::ClientType;
use crate::game::messages::server::ServerMessage;
use async_trait::async_trait;
use connection::Connection;

pub struct Client<'a> {
    pub entity: legion::Entity,
    pub connection: Connection<'a>,
    pub client_message_tx: crossbeam_channel::Sender<ClientMessage>,
    pub server_message_rx: tokio::sync::mpsc::UnboundedReceiver<ServerMessage>,
}

#[async_trait]
pub trait ProtocolClient {
    async fn run_client(&self, client: &mut Client) -> Result<(), ProtocolError>;
}

pub struct Protocol {
    pub client_type: ClientType,
    pub packet_codec: Box<dyn packet::PacketCodec + Send + Sync>,
    pub create_client: fn() -> Box<dyn ProtocolClient + Send + Sync>,
}

pub mod server;
