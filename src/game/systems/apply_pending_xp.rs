use legion::{system, systems::CommandBuffer, world::SubWorld, Query};

use crate::game::{
    components::{
        ClientEntity, Command, DamageSource, DamageSources, GameClient, HealthPoints, Level, NpcAi,
        Position,
    },
    messages::server::{DamageEntity, ServerMessage, UpdateXpStamina},
    resources::{DeltaTime, PendingXpList, ServerMessages},
};

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_pending_xp(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    entity_query: &mut Query<(Option<&GameClient>, &ClientEntity, &mut Level)>,
    source_entity_query: &mut Query<(&ClientEntity)>,
    #[resource] pending_xp_list: &mut PendingXpList,
    #[resource] server_messages: &mut ServerMessages,
) {
    let (source_entity_query_world, world) = world.split_for_query(source_entity_query);
    let mut entity_query_world = world;

    for pending_xp in pending_xp_list.iter() {
        if let Ok((client, client_entity, level)) =
            entity_query.get_mut(&mut entity_query_world, pending_xp.entity)
        {
            level.xp = level.xp.saturating_add(pending_xp.xp as u64);

            // TODO: Reward stamina

            // TODO: Handle level up

            // If not level up, then just send normal set xp packet
            if let Some(client) = client {
                let source_entity_id = pending_xp
                    .source
                    .and_then(|source_entity| {
                        source_entity_query
                            .get(&source_entity_query_world, source_entity)
                            .ok()
                    })
                    .map(|source_client_entity| source_client_entity.id.0);

                client
                    .server_message_tx
                    .send(ServerMessage::UpdateXpStamina(UpdateXpStamina {
                        xp: level.xp,
                        stamina: 0,
                        source_entity_id,
                    }))
                    .ok();
            }
        }
    }

    pending_xp_list.clear();
}
