use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};

use crate::{
    data::StatusEffectType,
    game::{
        components::{
            BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment,
            HealthPoints, Inventory, Level, ManaPoints, Npc, SkillList, StatusEffects,
        },
        messages::server::{ServerMessage, UpdateStatusEffects},
        resources::{ServerMessages, ServerTime},
        GameData,
    },
};

#[system]
pub fn status_effect(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    query: &mut Query<(
        Entity,
        &ClientEntity,
        &mut HealthPoints,
        &mut ManaPoints,
        &mut StatusEffects,
    )>,
    character_ability_values_query: &mut Query<(
        &CharacterInfo,
        &Level,
        &Equipment,
        &Inventory,
        &BasicStats,
        &SkillList,
    )>,
    npc_ability_values_query: &mut Query<&Npc>,
    #[resource] game_data: &GameData,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] server_time: &ServerTime,
) {
    let (character_ability_values_query_world, mut world) =
        world.split_for_query(character_ability_values_query);
    let (npc_ability_values_query_world, mut world) =
        world.split_for_query(npc_ability_values_query);

    for (entity, client_entity, health_points, mana_points, status_effects) in
        query.iter_mut(&mut world)
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
            let updated_ability_values =
                if matches!(client_entity.entity_type, ClientEntityType::Character) {
                    if let Ok((
                        character_info,
                        level,
                        equipment,
                        inventory,
                        basic_stats,
                        skill_list,
                    )) = character_ability_values_query
                        .get(&character_ability_values_query_world, *entity)
                    {
                        Some(game_data.ability_value_calculator.calculate(
                            character_info,
                            level,
                            equipment,
                            inventory,
                            basic_stats,
                            skill_list,
                        ))
                    } else {
                        None
                    }
                } else if let Ok(npc) =
                    npc_ability_values_query.get(&npc_ability_values_query_world, *entity)
                {
                    game_data.ability_value_calculator.calculate_npc(npc.id)
                } else {
                    None
                };

            if let Some(updated_ability_values) = updated_ability_values {
                health_points.hp = health_points
                    .hp
                    .max(updated_ability_values.max_health as u32);
                mana_points.mp = mana_points.mp.max(updated_ability_values.max_mana as u32);

                cmd.add_component(*entity, updated_ability_values);
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
