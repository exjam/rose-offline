use legion::*;

use crate::game::components::LoginClient;
use crate::game::messages::client::ClientMessage;
use crate::game::messages::server::{ConnectionResult, ServerMessage};

#[system(for_each)]
pub fn login_server(client: &mut LoginClient)
{
    if let Ok(message) = client.recv_message_rx.try_recv() {
        match message {
            ClientMessage::ConnectionRequest { .. } => {
                client.send_message_tx.send(ServerMessage::ConnectionReply {
                    status: ConnectionResult::Ok,
                    packet_sequence_id: 123,
                }).ok();
            },
        }
    }
}
