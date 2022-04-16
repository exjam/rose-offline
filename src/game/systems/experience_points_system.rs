use bevy::ecs::prelude::{Entity, EventReader, EventWriter, Query, Res, ResMut};

use crate::game::{
    components::{
        BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints, GameClient,
        HealthPoints, Level, ManaPoints, SkillList, SkillPoints, Stamina, StatPoints,
        StatusEffects, MAX_STAMINA,
    },
    events::{QuestTriggerEvent, RewardXpEvent},
    messages::server::{ServerMessage, UpdateLevel, UpdateXpStamina},
    resources::ServerMessages,
    GameData,
};

pub fn experience_points_system(
    mut entity_query: Query<(
        Entity,
        &ClientEntity,
        &mut Level,
        &mut ExperiencePoints,
        &mut Stamina,
        &mut SkillPoints,
        &mut StatPoints,
        Option<&GameClient>,
    )>,
    mut ability_values_query: Query<(
        &mut HealthPoints,
        &mut ManaPoints,
        &CharacterInfo,
        &Equipment,
        &BasicStats,
        &SkillList,
        &StatusEffects,
    )>,
    source_entity_query: Query<&ClientEntity>,
    game_data: Res<GameData>,
    mut quest_trigger_events: EventWriter<QuestTriggerEvent>,
    mut reward_xp_events: EventReader<RewardXpEvent>,
    mut server_messages: ResMut<ServerMessages>,
) {
    for reward_xp_event in reward_xp_events.iter() {
        if let Ok((
            entity,
            client_entity,
            mut level,
            mut experience_points,
            mut stamina,
            mut skill_points,
            mut stat_points,
            game_client,
        )) = entity_query.get_mut(reward_xp_event.entity)
        {
            experience_points.xp = experience_points
                .xp
                .saturating_add(reward_xp_event.xp as u64);

            stamina.stamina = stamina
                .stamina
                .saturating_add(reward_xp_event.stamina as u32);
            if stamina.stamina > MAX_STAMINA {
                stamina.stamina = MAX_STAMINA;
            }

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
                // Call every level up quest trigger
                for trigger_level in (level_before + 1)..=level.level {
                    quest_trigger_events.send(QuestTriggerEvent {
                        trigger_entity: entity,
                        trigger_hash: format!("levelup_{}", trigger_level).as_str().into(),
                    });
                }

                // Update ability values and restore hp / mp
                if let Ok((
                    mut health_points,
                    mut mana_points,
                    character_info,
                    equipment,
                    basic_stats,
                    skill_list,
                    status_effects,
                )) = ability_values_query.get_mut(entity)
                {
                    // Set to max hp / mana on levelup
                    let ability_values = game_data.ability_value_calculator.calculate(
                        character_info,
                        &level,
                        equipment,
                        basic_stats,
                        skill_list,
                        status_effects,
                    );

                    health_points.hp = ability_values.get_max_health();
                    mana_points.mp = ability_values.get_max_mana();
                }

                // Send level up packet
                server_messages.send_entity_message(
                    client_entity,
                    ServerMessage::UpdateLevel(UpdateLevel {
                        entity_id: client_entity.id,
                        level: level.clone(),
                        experience_points: experience_points.clone(),
                        stat_points: *stat_points,
                        skill_points: *skill_points,
                    }),
                );
            } else if let Some(game_client) = game_client {
                // If not level up, then just send normal set xp packet
                let source_entity_id = reward_xp_event
                    .source
                    .and_then(|source_entity| source_entity_query.get(source_entity).ok())
                    .map(|source_client_entity| source_client_entity.id);

                game_client
                    .server_message_tx
                    .send(ServerMessage::UpdateXpStamina(UpdateXpStamina {
                        xp: experience_points.xp,
                        stamina: stamina.stamina,
                        source_entity_id,
                    }))
                    .ok();
            }
        }
    }
}
