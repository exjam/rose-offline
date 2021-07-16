use legion::{systems::CommandBuffer, Entity};

use crate::game::{
    components::{
        ClientEntity, ClientEntityVisibility, Command, GameClient, NextCommand, Position,
    },
    messages::server::{ServerMessage, Teleport},
    resources::ClientEntityList,
};

mod ability_values;

pub use ability_values::{
    ability_values_add_value, ability_values_get_value, ability_values_set_value,
};

pub fn client_entity_leave_zone(
    cmd: &mut CommandBuffer,
    client_entity_list: &mut ClientEntityList,
    entity: &Entity,
    client_entity: &ClientEntity,
    position: &Position,
) {
    if let Some(client_entity_zone) = client_entity_list.get_zone_mut(position.zone as usize) {
        client_entity_zone.free(client_entity.id)
    }
    cmd.remove_component::<ClientEntity>(*entity);
    cmd.remove_component::<ClientEntityVisibility>(*entity);
}

pub fn client_entity_teleport_zone(
    cmd: &mut CommandBuffer,
    client_entity_list: &mut ClientEntityList,
    entity: &Entity,
    client_entity: &ClientEntity,
    previous_position: &Position,
    new_position: Position,
    game_client: Option<&GameClient>,
) {
    client_entity_leave_zone(
        cmd,
        client_entity_list,
        entity,
        client_entity,
        previous_position,
    );
    cmd.add_component(*entity, Command::with_stop());
    cmd.add_component(*entity, NextCommand::with_stop());
    cmd.add_component(*entity, new_position.clone());

    if let Some(game_client) = game_client {
        game_client
            .server_message_tx
            .send(ServerMessage::Teleport(Teleport {
                entity_id: client_entity.id,
                zone_no: new_position.zone,
                x: new_position.position.x,
                y: new_position.position.y,
                run_mode: 1,  // TODO: Run mode
                ride_mode: 0, // TODO: Ride mode
            }))
            .ok();
    }
}
