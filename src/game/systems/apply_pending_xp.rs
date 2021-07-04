use legion::{system, world::SubWorld, Entity, Query};

use crate::game::{
    components::{ClientEntity, ExperiencePoints, GameClient, Level, SkillPoints, StatPoints},
    messages::server::{ServerMessage, UpdateLevel, UpdateXpStamina},
    resources::{PendingXpList, ServerMessages},
    GameData,
};

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_pending_xp(
    world: &mut SubWorld,
    entity_query: &mut Query<(
        Entity,
        &ClientEntity,
        &mut Level,
        &mut ExperiencePoints,
        Option<&mut SkillPoints>,
        Option<&mut StatPoints>,
        Option<&GameClient>,
    )>,
    source_entity_query: &mut Query<&ClientEntity>,
    #[resource] game_data: &GameData,
    #[resource] pending_xp_list: &mut PendingXpList,
    #[resource] server_messages: &mut ServerMessages,
) {
    let (source_entity_query_world, world) = world.split_for_query(source_entity_query);
    let mut entity_query_world = world;

    for pending_xp in pending_xp_list.iter() {
        if let Ok((
            entity,
            client_entity,
            level,
            experience_points,
            skill_points,
            stat_points,
            game_client,
        )) = entity_query.get_mut(&mut entity_query_world, pending_xp.entity)
        {
            experience_points.xp = experience_points.xp.saturating_add(pending_xp.xp as u64);

            // TODO: Reward stamina
            // TODO: Apply level cap
            // TODO: Penalty xp?

            let level_before = level.level;
            loop {
                let need_xp = game_data
                    .ability_value_calculator
                    .calculate_levelup_require_xp(level.level);
                if experience_points.xp < need_xp {
                    break;
                }

                experience_points.xp -= need_xp;
                level.level += 1;

                if let Some(&mut ref mut skill_points) = skill_points {
                    (*skill_points).points += game_data
                        .ability_value_calculator
                        .calculate_levelup_reward_skill_points(level.level);
                }

                if let Some(&mut ref mut stat_points) = stat_points {
                    (*stat_points).points += game_data
                        .ability_value_calculator
                        .calculate_levelup_reward_stat_points(level.level);
                }
            }

            if level.level != level_before {
                // Send level up packet

                // TODO: Update ability values
                // TODO: Restore hp / mp
                // TODO: Call level up quest trigger

                server_messages.send_entity_message(
                    *entity,
                    ServerMessage::UpdateLevel(UpdateLevel {
                        entity_id: client_entity.id,
                        level: level.clone(),
                        experience_points: experience_points.clone(),
                        stat_points: stat_points.cloned().unwrap_or_else(StatPoints::new),
                        skill_points: skill_points.cloned().unwrap_or_else(SkillPoints::new),
                    }),
                );
            } else if let Some(game_client) = game_client {
                // If not level up, then just send normal set xp packet
                let source_entity_id = pending_xp
                    .source
                    .and_then(|source_entity| {
                        source_entity_query
                            .get(&source_entity_query_world, source_entity)
                            .ok()
                    })
                    .map(|source_client_entity| source_client_entity.id);

                game_client
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
