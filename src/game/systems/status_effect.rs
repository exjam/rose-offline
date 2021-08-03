use bevy_ecs::prelude::{Commands, Entity, Query, Res, ResMut};

use crate::{
    data::StatusEffectType,
    game::{
        bundles::client_entity_recalculate_ability_values,
        components::{
            BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment, HealthPoints,
            Level, ManaPoints, MoveMode, Npc, SkillList, StatusEffects,
        },
        messages::server::{ServerMessage, UpdateStatusEffects},
        resources::{ServerMessages, ServerTime},
        GameData,
    },
};

pub fn status_effect_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &ClientEntity,
        &mut HealthPoints,
        &mut ManaPoints,
        &mut StatusEffects,
    )>,
    character_ability_values_query: Query<(
        &CharacterInfo,
        &Level,
        &Equipment,
        &BasicStats,
        &SkillList,
        &MoveMode,
    )>,
    npc_ability_values_query: Query<(&Npc, &Level, &MoveMode)>,
    game_data: Res<GameData>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    for (entity, client_entity, mut health_points, mut mana_points, mut status_effects) in
        query.iter_mut()
    {
        let mut status_effects_expired = false;
        let mut cleared_hp = false;
        let mut cleared_mp = false;

        // TODO: Process per tick status effect: IncreaseHp, IncreaseMp, Poisoned, DecreaseLifeTime

        for (status_effect_type, status_effect_slot) in status_effects.active.iter_mut() {
            if let Some(status_effect) = status_effect_slot {
                if status_effect.time_remaining > server_time.delta {
                    status_effect.time_remaining -= server_time.delta;
                } else {
                    status_effects_expired = true;
                    *status_effect_slot = None;

                    match status_effect_type {
                        StatusEffectType::IncreaseHp => cleared_hp = true,
                        StatusEffectType::IncreaseMp => cleared_mp = true,
                        _ => {}
                    }
                }
            }
        }

        if status_effects_expired {
            // Update ability values
            if matches!(client_entity.entity_type, ClientEntityType::Character) {
                if let Ok((character_info, level, equipment, basic_stats, skill_list, move_mode)) =
                    character_ability_values_query.get(entity)
                {
                    client_entity_recalculate_ability_values(
                        &mut commands,
                        game_data.ability_value_calculator.as_ref(),
                        client_entity,
                        entity,
                        &status_effects,
                        Some(basic_stats),
                        Some(character_info),
                        Some(equipment),
                        Some(level),
                        Some(move_mode),
                        Some(skill_list),
                        None,
                        Some(&mut health_points),
                        Some(&mut mana_points),
                    );
                }
            } else if let Ok((npc, level, move_mode)) = npc_ability_values_query.get(entity) {
                client_entity_recalculate_ability_values(
                    &mut commands,
                    game_data.ability_value_calculator.as_ref(),
                    client_entity,
                    entity,
                    &status_effects,
                    None,
                    None,
                    None,
                    Some(level),
                    Some(move_mode),
                    None,
                    Some(npc),
                    Some(&mut health_points),
                    Some(&mut mana_points),
                );
            }

            // Send status effect expiry message
            server_messages.send_entity_message(
                client_entity,
                ServerMessage::UpdateStatusEffects(UpdateStatusEffects {
                    entity_id: client_entity.id,
                    status_effects: status_effects.clone(),
                    updated_hp: if cleared_hp {
                        Some(*health_points)
                    } else {
                        None
                    },
                    updated_mp: if cleared_mp { Some(*mana_points) } else { None },
                }),
            );
        }
    }
}
