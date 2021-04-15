use crate::game::messages::{client::ClientMessage, server::ServerMessage};
use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;

pub struct GameClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
    pub login_token: u32,
}

impl GameClient {
    pub fn new(
        client_message_rx: Receiver<ClientMessage>,
        server_message_tx: UnboundedSender<ServerMessage>,
    ) -> Self {
        Self {
            client_message_rx,
            server_message_tx,
            login_token: 0u32,
        }
    }
}
