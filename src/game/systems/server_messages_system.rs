use bevy::ecs::prelude::{Query, ResMut};

use crate::game::{
    components::{ClientEntityVisibility, GameClient, Position},
    resources::ServerMessages,
};

pub fn server_messages_system(
    query: Query<(&GameClient, &Position, &ClientEntityVisibility)>,
    mut server_messages: ResMut<ServerMessages>,
) {
    for (game_client, position, client_visibility) in query.iter() {
        for message in server_messages.pending_global_messages.iter() {
            game_client
                .server_message_tx
                .send(message.message.clone())
                .ok();
        }

        for message in server_messages.pending_zone_messages.iter() {
            if position.zone_id == message.zone_id {
                game_client
                    .server_message_tx
                    .send(message.message.clone())
                    .ok();
            }
        }

        for message in server_messages.pending_entity_messages.iter() {
            if position.zone_id == message.zone_id
                && client_visibility
                    .get(message.entity_id.0)
                    .map_or(false, |b| *b)
            {
                game_client
                    .server_message_tx
                    .send(message.message.clone())
                    .ok();
            }
        }
    }

    server_messages.pending_global_messages.clear();
    server_messages.pending_zone_messages.clear();
    server_messages.pending_entity_messages.clear();
}
