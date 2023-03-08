use bevy::ecs::prelude::{Commands, Entity, Query, Res, ResMut};
use bevy::math::Vec3Swizzles;
use bevy::time::Time;

use crate::game::{
    components::{ClientEntity, ClientEntitySector, Destination, MoveSpeed, Position},
    resources::ClientEntityList,
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
    time: Res<Time>,
) {
    query.for_each_mut(
        |(entity, client_entity, client_entity_sector, move_speed, mut position, destination)| {
            let direction = destination.xy() - position.position.xy();
            let distance_squared = direction.length_squared();

            if distance_squared == 0.0 {
                position.position = **destination;
                commands.entity(entity).remove::<Destination>();
            } else {
                let move_vector = direction.normalize() * move_speed.speed * time.delta_seconds();
                if move_vector.length_squared() >= distance_squared {
                    position.position = **destination;
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
