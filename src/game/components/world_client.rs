use crate::game::messages::{client::ClientMessage, server::ServerMessage};
use crossbeam_channel::Receiver;
use legion::Entity;
use tokio::sync::mpsc::UnboundedSender;

pub struct WorldClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
    pub login_token: u32,
    pub selected_game_server: Option<Entity>,
}

impl WorldClient {
    pub fn new(
        client_message_rx: Receiver<ClientMessage>,
        server_message_tx: UnboundedSender<ServerMessage>,
    ) -> Self {
        Self {
            client_message_rx,
            server_message_tx,
            login_token: 0u32,
            selected_game_server: None,
        }
    }
}
