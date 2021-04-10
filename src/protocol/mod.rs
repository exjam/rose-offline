#[derive(Debug)]
pub enum ProtocolError {
    Disconnect,
    IscError,
    InvalidPacket,
}

mod packet;

use crate::game::messages::control::ClientType;

pub struct Protocol {
    pub client_type: ClientType,
    pub packet_codec: Box<dyn packet::PacketCodec + Send + Sync>,
    pub packet_encoder: Box<dyn packet::PacketEncoder + Send + Sync>,
    pub packet_decoder: Box<dyn packet::PacketDecoder + Send + Sync>,
}

mod connection;

use connection::Connection;

pub mod irose;

pub mod server;
