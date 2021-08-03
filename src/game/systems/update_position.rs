use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};

use crate::game::{
    components::{ClientEntity, Destination, MoveSpeed, Position},
    resources::{ClientEntityList, ServerTime},
};

pub fn update_position_system(
    mut commands: Commands,
    query: Query<(
        Entity,
        Option<&mut ClientEntity>,
        &MoveSpeed,
        &mut Position,
        &Destination,
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    server_time: Res<ServerTime>,
) {
    query.for_each_mut(
        |(entity, client_entity, move_speed, mut position, destination)| {
            let direction = destination.position.xy() - position.position.xy();
            let distance_squared = direction.magnitude_squared();

            if distance_squared == 0.0 {
                position.position = destination.position;
                commands.entity(entity).remove::<Destination>();
            } else {
                let move_vector =
                    direction.normalize() * move_speed.speed * server_time.delta.as_secs_f32();
                if move_vector.magnitude_squared() >= distance_squared {
                    position.position = destination.position;
                    commands.entity(entity).remove::<Destination>();
                } else {
                    position.position.x += move_vector.x;
                    position.position.y += move_vector.y;
                }
            }

            if let Some(mut client_entity) = client_entity {
                if let Some(zone) = client_entity_list.get_zone_mut(position.zone_id) {
                    zone.update_position(entity, &mut client_entity, position.position)
                }
            }
        },
    );
}
