use crossbeam_channel::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use crate::game::messages::{client::ClientMessage, server::ServerMessage};

pub struct GameClient
{
    pub recv_message_rx: Receiver<ClientMessage>,
    pub send_message_tx: UnboundedSender<ServerMessage>,
}
