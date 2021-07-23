use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{ClientEntity, Command, ExpireTime, Position},
    resources::{ClientEntityList, ServerTime},
};

#[allow(clippy::clippy::type_complexity)]
#[system]
pub fn expire_time(
    cmd: &mut CommandBuffer,
    world: &SubWorld,
    query: &mut Query<(
        Entity,
        &ExpireTime,
        Option<&Position>,
        Option<&ClientEntity>,
        Option<&Command>,
    )>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] server_time: &mut ServerTime,
) {
    query.for_each(
        world,
        |(entity, expire_time, position, client_entity, command)| {
            if server_time.now >= expire_time.when {
                if command.is_some() {
                    cmd.add_component(*entity, Command::with_die(None, None));
                } else {
                    if let (Some(position), Some(client_entity)) = (position, client_entity) {
                        client_entity_leave_zone(
                            cmd,
                            client_entity_list,
                            entity,
                            client_entity,
                            position,
                        );
                    }
                    cmd.remove(*entity);
                }
            }
        },
    );
}
