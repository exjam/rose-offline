use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{ClientEntity, Command, ExpireTime, Position},
    resources::{ClientEntityList, ServerTime},
};

#[allow(clippy::clippy::type_complexity)]
pub fn expire_time_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        &ExpireTime,
        Option<&Position>,
        Option<&ClientEntity>,
        Option<&Command>,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    query.for_each(|(entity, expire_time, position, client_entity, command)| {
        if server_time.now >= expire_time.when {
            if command.is_some() {
                commands
                    .entity(entity)
                    .insert(Command::with_die(None, None));
            } else {
                if let (Some(position), Some(client_entity)) = (position, client_entity) {
                    client_entity_leave_zone(
                        &mut commands,
                        &mut client_entity_list,
                        entity,
                        client_entity,
                        position,
                    );
                }
                commands.entity(entity).despawn();
            }
        }
    });
}
