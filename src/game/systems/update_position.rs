use legion::{system, systems::CommandBuffer, Entity};

use crate::game::{
    components::{ClientEntity, Destination, MoveSpeed, Position},
    resources::{ClientEntityList, DeltaTime},
};

#[system(for_each)]
pub fn update_position(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client_entity: Option<&mut ClientEntity>,
    move_speed: &MoveSpeed,
    position: &mut Position,
    destination: &Destination,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] delta_time: &DeltaTime,
) {
    let direction = destination.position - position.position;
    let distance_squared = direction.magnitude_squared();

    if distance_squared == 0.0 {
        position.position = destination.position;
        cmd.remove_component::<Destination>(*entity);
    } else {
        let move_vector = direction.normalize() * move_speed.speed * delta_time.delta.as_secs_f32();
        if move_vector.magnitude_squared() >= distance_squared {
            position.position = destination.position;
            cmd.remove_component::<Destination>(*entity);
        } else {
            position.position += move_vector;
        }
    }

    if let Some(client_entity) = client_entity {
        client_entity_list
            .get_zone_mut(position.zone as usize)
            .map(|zone| zone.update_sector(entity, client_entity, position.position));
    }
}
