use crate::game::messages::{client::ClientMessage, server::ServerMessage};
use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;

pub struct WorldClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
}
