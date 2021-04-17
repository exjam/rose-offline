use crossbeam_channel::Receiver;
use legion::Entity;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use super::{client::ClientMessage, server::ServerMessage};

#[derive(Clone, Copy)]
pub enum ClientType {
    Login,
    World,
    Game,
}

pub enum ControlMessage {
    AddClient {
        client_type: ClientType,
        client_message_rx: Receiver<ClientMessage>,
        server_message_tx: UnboundedSender<ServerMessage>,
        response_tx: oneshot::Sender<Entity>,
    },
    RemoveClient {
        client_type: ClientType,
        entity: Entity,
    },
    AddWorldServer {
        name: String,
        ip: String,
        port: u16,
        packet_codec_seed: u32, // TODO: Make this protocol agnostic data ? Might need something different for different game versions
        response_tx: oneshot::Sender<Entity>,
    },
    AddGameServer {
        world_server: Entity,
        name: String,
        ip: String,
        port: u16,
        packet_codec_seed: u32,
        response_tx: oneshot::Sender<Entity>,
    },
    RemoveServer {
        entity: Entity,
    },
}
