use legion::world::SubWorld;
use legion::*;

use crate::game::components::{GameClient, Position};
use crate::game::resources::ServerMessages;

// TODO: Read sector size from zone STB for how we define "nearby"
const NEARBY_DISTANCE: f32 = 10000f32;

#[system]
pub fn server_messages_sender(
    world: &SubWorld,
    query: &mut Query<(Entity, &Position, &GameClient)>,
    #[resource] server_messages: &mut ServerMessages,
) {
    for (entity, position, client) in query.iter(world) {
        for message in server_messages.pending_global_messages.iter() {
            client.server_message_tx.send(message.message.clone()).ok();
        }

        for message in server_messages.pending_zone_messages.iter() {
            if position.zone == message.zone {
                client.server_message_tx.send(message.message.clone()).ok();
            }
        }

        for message in server_messages.pending_nearby_messages.iter() {
            if message.except_entity.is_none() || message.except_entity.as_ref().unwrap() != entity
            {
                if position
                    .position
                    .metric_distance(&message.position.position)
                    < NEARBY_DISTANCE
                {
                    client.server_message_tx.send(message.message.clone()).ok();
                }
            }
        }
    }

    server_messages.pending_global_messages.clear();
    server_messages.pending_zone_messages.clear();
    server_messages.pending_nearby_messages.clear();
}
