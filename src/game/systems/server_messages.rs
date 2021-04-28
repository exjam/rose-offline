use legion::world::SubWorld;
use legion::*;

use crate::game::components::{ClientEntityVisibility, GameClient, Position};
use crate::game::resources::ServerMessages;

#[system]
pub fn server_messages_sender(
    world: &SubWorld,
    query: &mut Query<(&Position, &GameClient, &ClientEntityVisibility)>,
    #[resource] server_messages: &mut ServerMessages,
) {
    for (position, client, client_visibility) in query.iter(world) {
        for message in server_messages.pending_global_messages.iter() {
            client.server_message_tx.send(message.message.clone()).ok();
        }

        for message in server_messages.pending_zone_messages.iter() {
            if position.zone == message.zone {
                client.server_message_tx.send(message.message.clone()).ok();
            }
        }

        for message in server_messages.pending_entity_messages.iter() {
            if client_visibility.entities.contains(&message.entity) {
                client.server_message_tx.send(message.message.clone()).ok();
            }
        }
    }

    server_messages.pending_global_messages.clear();
    server_messages.pending_zone_messages.clear();
    server_messages.pending_entity_messages.clear();
}
