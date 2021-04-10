
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
        recv_message_rx: Receiver<ClientMessage>,
        send_message_tx: UnboundedSender<ServerMessage>,
        response_tx: oneshot::Sender<Entity>,
    },
    RemoveClient {
        entity: Entity,
    },
}
