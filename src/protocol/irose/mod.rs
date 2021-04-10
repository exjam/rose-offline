mod packet_codec;
use std::sync::Arc;

use packet_codec::PacketCodec;

use crate::game::messages::control::ClientType;
use crate::protocol::Protocol;

mod game;
mod login;
mod world;

pub fn login_protocol() -> Arc<Protocol> {
    Arc::new(Protocol {
        client_type: ClientType::Login,
        packet_codec: Box::new(PacketCodec::default(&packet_codec::IROSE_112_TABLE)),
        packet_encoder: Box::new(login::LoginPacketEncoder::new()),
        packet_decoder: Box::new(login::LoginPacketDecoder::new()),
    })
}

pub fn world_protocol() -> Arc<Protocol> {
    let packet_codec_seed = 0x12345678; // This can be any non-zero value
    Arc::new(Protocol {
        client_type: ClientType::World,
        packet_codec: Box::new(PacketCodec::init(
            &packet_codec::IROSE_112_TABLE,
            packet_codec_seed,
        )),
        packet_encoder: Box::new(world::WorldPacketEncoder::new()),
        packet_decoder: Box::new(world::WorldPacketDecoder::new()),
    })
}

pub fn game_protocol() -> Arc<Protocol> {
    let packet_codec_seed = 0x87654321; // This can be any non-zero value
    Arc::new(Protocol {
        client_type: ClientType::Game,
        packet_codec: Box::new(PacketCodec::init(
            &packet_codec::IROSE_112_TABLE,
            packet_codec_seed,
        )),
        packet_encoder: Box::new(world::WorldPacketEncoder::new()),
        packet_decoder: Box::new(world::WorldPacketDecoder::new()),
    })
}
