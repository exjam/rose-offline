use bevy::ecs::prelude::{Component, Entity};
use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;

use crate::game::messages::{client::ClientMessage, server::ServerMessage};

#[derive(Component)]
pub struct WorldClient {
    pub client_message_rx: Receiver<ClientMessage>,
    pub server_message_tx: UnboundedSender<ServerMessage>,
    pub login_token: u32,
    pub selected_game_server: Option<Entity>,
    pub game_client_entity: Option<Entity>,
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
            game_client_entity: None,
        }
    }
}
