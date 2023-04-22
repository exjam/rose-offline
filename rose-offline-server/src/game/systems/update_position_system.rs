use bevy::{
    ecs::prelude::{Entity, Query, Res, ResMut},
    math::Vec3Swizzles,
    time::Time,
};

use crate::game::{
    components::{ClientEntity, ClientEntitySector, Command, CommandData, MoveSpeed, Position},
    resources::ClientEntityList,
};

pub fn update_position_system(
    mut query: Query<(
        Entity,
        Option<&ClientEntity>,
        Option<&mut ClientEntitySector>,
        &MoveSpeed,
        &mut Position,
        &Command,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    time: Res<Time>,
) {
    query.for_each_mut(
        |(entity, client_entity, client_entity_sector, move_speed, mut position, command)| {
            let CommandData::Move {
                destination,
                ..
            } = command.command else {
                return;
            };

            let direction = destination.xy() - position.position.xy();
            let distance_squared = direction.length_squared();

            if distance_squared == 0.0 {
                position.position = destination;
            } else {
                let move_vector = direction.normalize() * move_speed.speed * time.delta_seconds();
                if move_vector.length_squared() >= distance_squared {
                    position.position = destination;
                } else {
                    position.position.x += move_vector.x;
                    position.position.y += move_vector.y;
                }
            }

            if let (Some(client_entity), Some(mut client_entity_sector)) =
                (client_entity, client_entity_sector)
            {
                if let Some(zone) = client_entity_list.get_zone_mut(position.zone_id) {
                    zone.update_position(
                        entity,
                        client_entity,
                        &mut client_entity_sector,
                        position.position,
                    )
                }
            }
        },
    );
}
