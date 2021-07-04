use legion::{system, world::SubWorld, Query};

use crate::game::{
    components::{ClientEntity, ExperiencePoints, GameClient, Level},
    messages::server::{ServerMessage, UpdateXpStamina},
    resources::PendingXpList,
};

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_pending_xp(
    world: &mut SubWorld,
    entity_query: &mut Query<(
        Option<&GameClient>,
        &ClientEntity,
        &mut Level,
        &mut ExperiencePoints,
    )>,
    source_entity_query: &mut Query<&ClientEntity>,
    #[resource] pending_xp_list: &mut PendingXpList,
) {
    let (source_entity_query_world, world) = world.split_for_query(source_entity_query);
    let mut entity_query_world = world;

    for pending_xp in pending_xp_list.iter() {
        if let Ok((client, _client_entity, _level, experience_points)) =
            entity_query.get_mut(&mut entity_query_world, pending_xp.entity)
        {
            experience_points.xp = experience_points.xp.saturating_add(pending_xp.xp as u64);

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
                        xp: experience_points.xp,
                        stamina: 0,
                        source_entity_id,
                    }))
                    .ok();
            }
        }
    }

    pending_xp_list.clear();
}
