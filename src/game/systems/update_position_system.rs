use bevy::ecs::prelude::{Commands, Entity, Query, Res, ResMut};
use bevy::math::Vec3Swizzles;

use crate::game::{
    components::{ClientEntity, ClientEntitySector, Destination, MoveSpeed, Position},
    resources::{ClientEntityList, ServerTime},
};

pub fn update_position_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        Option<&ClientEntity>,
        Option<&mut ClientEntitySector>,
        &MoveSpeed,
        &mut Position,
        &Destination,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    query.for_each_mut(
        |(entity, client_entity, client_entity_sector, move_speed, mut position, destination)| {
            let direction = destination.position.xy() - position.position.xy();
            let distance_squared = direction.length_squared();

            if distance_squared == 0.0 {
                position.position = destination.position;
                commands.entity(entity).remove::<Destination>();
            } else {
                let move_vector =
                    direction.normalize() * move_speed.speed * server_time.delta.as_secs_f32();
                if move_vector.length_squared() >= distance_squared {
                    position.position = destination.position;
                    commands.entity(entity).remove::<Destination>();
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
