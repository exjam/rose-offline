use legion::systems::CommandBuffer;
use legion::*;

use crate::game::components::{Destination, MoveSpeed, Position};
use crate::game::resources::DeltaTime;

#[system(for_each)]
pub fn update_position(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    move_speed: &MoveSpeed,
    position: &mut Position,
    destination: &Destination,
    #[resource] delta_time: &DeltaTime,
) {
    let direction = destination.position - position.position;
    if direction.magnitude_squared() == 0.0 {
        position.position = destination.position;
        cmd.remove_component::<Destination>(*entity);
        return;
    }

    let move_vector = direction.normalize() * move_speed.speed * delta_time.delta.as_secs_f32();
    if move_vector.magnitude_squared() >= direction.magnitude_squared() {
        position.position = destination.position;
        cmd.remove_component::<Destination>(*entity);
        return;
    }

    position.position += move_vector;
}
