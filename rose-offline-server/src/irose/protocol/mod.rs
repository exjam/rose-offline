use std::sync::Arc;

use rose_network_irose::{ServerPacketCodec, IROSE_112_TABLE};

use crate::{game::messages::control::ClientType, protocol::Protocol};

mod game_server;
mod login_server;
mod world_server;

use game_server::GameServer;
use login_server::LoginServer;
use world_server::WorldServer;

pub fn login_protocol() -> Arc<Protocol> {
    Arc::new(Protocol {
        client_type: ClientType::Login,
        packet_codec: Box::new(ServerPacketCodec::default(&IROSE_112_TABLE)),
        create_server: || Box::new(LoginServer::new()),
    })
}

pub fn world_protocol() -> Arc<Protocol> {
    let packet_codec_seed = 0x12345678; // This can be any non-zero value
    Arc::new(Protocol {
        client_type: ClientType::World,
        packet_codec: Box::new(ServerPacketCodec::init(&IROSE_112_TABLE, packet_codec_seed)),
        create_server: || Box::new(WorldServer::new()),
    })
}

pub fn game_protocol() -> Arc<Protocol> {
    let packet_codec_seed = 0x87654321; // This can be any non-zero value
    Arc::new(Protocol {
        client_type: ClientType::Game,
        packet_codec: Box::new(ServerPacketCodec::init(&IROSE_112_TABLE, packet_codec_seed)),
        create_server: || Box::new(GameServer::new()),
    })
}
