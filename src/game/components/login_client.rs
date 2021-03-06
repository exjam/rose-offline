use bevy::ecs::prelude::Component;
use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;

use rose_game_common::messages::{client::ClientMessage, server::ServerMessage};

#[derive(Component)]
pub struct LoginClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
    pub login_token: u32,
}

impl LoginClient {
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
