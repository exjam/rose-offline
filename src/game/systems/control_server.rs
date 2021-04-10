use legion::*;
use legion::systems::CommandBuffer;
use crate::game::components::{ControlClient, LoginClient, WorldClient, GameClient};
use crate::game::messages::control::{ControlMessage, ClientType};

#[system]
pub fn control_server(cmd: &mut CommandBuffer, #[resource] client: &ControlClient) {
    loop {
        match client.control_rx.try_recv() {
            Ok(message) => match message {
                ControlMessage::AddClient {
                    client_type,
                    send_message_tx,
                    recv_message_rx,
                    response_tx,
                } => {
                    let entity = match client_type {
                        ClientType::Login => cmd.push((LoginClient { send_message_tx, recv_message_rx },)),
                        ClientType::World => cmd.push((WorldClient { send_message_tx, recv_message_rx },)),
                        ClientType::Game => cmd.push((GameClient { send_message_tx, recv_message_rx },)),
                    };
                    response_tx.send(entity).unwrap();
                }
                ControlMessage::RemoveClient { entity } => {
                    cmd.remove(entity);
                },
            },
            Err(_) => break,
        }
    }
}
