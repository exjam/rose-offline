use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::game::{
    components::{Command, ExpireTime},
    resources::ServerTime,
};

#[system]
pub fn expire_time(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    query: &mut Query<(Entity, &ExpireTime, Option<&Command>)>,
    #[resource] server_time: &mut ServerTime,
) {
    query.for_each(world, |(entity, expire_time, command)| {
        if server_time.now >= expire_time.when {
            if command.is_some() {
                cmd.add_component(*entity, Command::with_die(None));
            } else {
                cmd.remove(*entity);
            }
        }
    });
}
