use std::collections::VecDeque;

use crate::game::messages::{client::ClientMessage, server::ServerMessage};
use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;

pub struct WorldClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
    pub pending_messages: VecDeque<ClientMessage>,
    pub login_token: u32,
}

impl WorldClient {
    pub fn new(
        client_message_rx: Receiver<ClientMessage>,
        server_message_tx: UnboundedSender<ServerMessage>,
    ) -> Self {
        Self {
            client_message_rx,
            server_message_tx,
            pending_messages: VecDeque::new(),
            login_token: 0u32,
        }
    }
}
