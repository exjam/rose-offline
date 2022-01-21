use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{
        ClientEntity, ClientEntitySector, Command, EntityExpireTime, Owner, OwnerExpireTime,
        Position,
    },
    resources::{ClientEntityList, ServerTime},
};

pub fn expire_time_system(
    mut commands: Commands,
    entity_expire_time_query: Query<(
        Entity,
        &EntityExpireTime,
        Option<&Position>,
        Option<&ClientEntity>,
        Option<&ClientEntitySector>,
        Option<&Command>,
    )>,
    owner_expire_time_query: Query<(Entity, &OwnerExpireTime)>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    entity_expire_time_query.for_each(
        |(entity, entity_expire_time, position, client_entity, client_entity_sector, command)| {
            if server_time.now >= entity_expire_time.when {
                if command.is_some() {
                    commands
                        .entity(entity)
                        .insert(Command::with_die(None, None, None));
                } else {
                    if let (Some(position), Some(client_entity), Some(client_entity_sector)) =
                        (position, client_entity, client_entity_sector)
                    {
                        client_entity_leave_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            client_entity,
                            client_entity_sector,
                            position,
                        );
                    }
                    commands.entity(entity).despawn();
                }
            }
        },
    );

    owner_expire_time_query.for_each(|(entity, owner_expire_time)| {
        if server_time.now >= owner_expire_time.when {
            commands.entity(entity).remove::<Owner>();
        }
    });
}
