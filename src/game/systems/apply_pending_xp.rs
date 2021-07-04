use legion::{system, world::SubWorld, Entity, Query};

use crate::game::{
    components::{
        AbilityValues, BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints,
        GameClient, HealthPoints, Inventory, Level, ManaPoints, SkillList, SkillPoints, StatPoints,
    },
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
        &mut SkillPoints,
        &mut StatPoints,
        Option<&GameClient>,
    )>,
    ability_values_query: &mut Query<(
        &mut AbilityValues,
        &mut HealthPoints,
        &mut ManaPoints,
        &CharacterInfo,
        &Equipment,
        &Inventory,
        &BasicStats,
        &SkillList,
    )>,
    source_entity_query: &mut Query<&ClientEntity>,
    #[resource] game_data: &GameData,
    #[resource] pending_xp_list: &mut PendingXpList,
    #[resource] server_messages: &mut ServerMessages,
) {
    let (mut ability_values_query_world, mut world) = world.split_for_query(ability_values_query);
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

                level.level += 1;
                experience_points.xp -= need_xp;

                skill_points.points += game_data
                    .ability_value_calculator
                    .calculate_levelup_reward_skill_points(level.level);

                stat_points.points += game_data
                    .ability_value_calculator
                    .calculate_levelup_reward_stat_points(level.level);
            }

            if level.level != level_before {
                // TODO: Call level up quest trigger

                // Update ability values and restore hp / mp
                if let Ok((
                    ability_values,
                    health_points,
                    mana_points,
                    character_info,
                    equipment,
                    inventory,
                    basic_stats,
                    skill_list,
                )) = ability_values_query.get_mut(&mut ability_values_query_world, *entity)
                {
                    *ability_values = game_data.ability_value_calculator.calculate(
                        character_info,
                        level,
                        equipment,
                        inventory,
                        basic_stats,
                        skill_list,
                    );
                    health_points.hp = ability_values.max_health as u32;
                    mana_points.mp = ability_values.max_mana as u32;
                }

                // Send level up packet
                server_messages.send_entity_message(
                    *entity,
                    ServerMessage::UpdateLevel(UpdateLevel {
                        entity_id: client_entity.id,
                        level: level.clone(),
                        experience_points: experience_points.clone(),
                        stat_points: stat_points.clone(),
                        skill_points: skill_points.clone(),
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
