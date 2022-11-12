use bevy::ecs::{
    entity::Entity,
    event::EventWriter,
    prelude::{Query, Res, ResMut},
};
use enum_map::EnumMap;
use std::time::Duration;

use rose_data::StatusEffectType;
use rose_game_common::data::Damage;

use crate::game::{
    components::{
        AbilityValues, ActiveStatusEffectRegen, ClientEntity, HealthPoints, ManaPoints,
        StatusEffects, StatusEffectsRegen,
    },
    events::DamageEvent,
    messages::server::{ServerMessage, UpdateStatusEffects},
    resources::{ServerMessages, ServerTime},
    GameData,
};

fn update_status_effect_regen(
    regen: &mut ActiveStatusEffectRegen,
    server_time: &ServerTime,
) -> i32 {
    let prev_applied_value = regen.applied_value;

    regen.applied_duration += server_time.delta;
    regen.applied_value = i32::min(
        ((regen.applied_duration.as_micros() as f32 / 1000000.0) * regen.value_per_second as f32)
            as i32,
        regen.total_value,
    );

    regen.applied_value - prev_applied_value
}

pub fn status_effect_system(
    mut query: Query<(
        Entity,
        &ClientEntity,
        &AbilityValues,
        &mut HealthPoints,
        Option<&mut ManaPoints>,
        &mut StatusEffects,
        &mut StatusEffectsRegen,
    )>,
    mut damage_events: EventWriter<DamageEvent>,
    mut server_messages: ResMut<ServerMessages>,
    game_data: Res<GameData>,
    server_time: Res<ServerTime>,
) {
    for (
        entity,
        client_entity,
        ability_values,
        mut health_points,
        mut mana_points,
        mut status_effects,
        mut status_effects_regen,
    ) in query.iter_mut()
    {
        let mut expired_status_effects: EnumMap<StatusEffectType, bool> = Default::default();
        let apply_per_second_effect = {
            status_effects_regen.per_second_tick_counter += server_time.delta;
            if status_effects_regen.per_second_tick_counter > Duration::from_secs(1) {
                status_effects_regen.per_second_tick_counter -= Duration::from_secs(1);
                true
            } else {
                false
            }
        };

        for (status_effect_type, status_effect_slot) in status_effects.active.iter() {
            if let Some(status_effect) = status_effect_slot {
                match status_effect_type {
                    StatusEffectType::IncreaseHp => {
                        if let Some(status_effect_regen) =
                            &mut status_effects_regen.regens[status_effect_type]
                        {
                            // Calculate regen for this tick
                            let regen =
                                update_status_effect_regen(status_effect_regen, &server_time);

                            // Update hp
                            let max_hp = ability_values.get_max_health();
                            health_points.hp = i32::min(health_points.hp + regen, max_hp);

                            // Expire when reach max hp
                            if health_points.hp == max_hp {
                                expired_status_effects[status_effect_type] = true;
                            }
                        }
                    }
                    StatusEffectType::IncreaseMp => {
                        if let Some(status_effect_regen) =
                            &mut status_effects_regen.regens[status_effect_type]
                        {
                            if let Some(mana_points) = mana_points.as_mut() {
                                // Calculate regen for this tick
                                let regen =
                                    update_status_effect_regen(status_effect_regen, &server_time);

                                // Update mp
                                let max_mp = ability_values.get_max_mana();
                                mana_points.mp = i32::min(mana_points.mp + regen, max_mp);

                                // Expire when reach max mp
                                if mana_points.mp == max_mp {
                                    expired_status_effects[status_effect_type] = true;
                                }
                            }
                        }
                    }
                    StatusEffectType::Poisoned => {
                        if apply_per_second_effect {
                            if let Some(data) =
                                game_data.status_effects.get_status_effect(status_effect.id)
                            {
                                health_points.hp =
                                    i32::max(health_points.hp - data.apply_per_second_value, 1);
                            }
                        }
                    }
                    StatusEffectType::DecreaseLifeTime => {
                        if apply_per_second_effect {
                            if let Some(data) =
                                game_data.status_effects.get_status_effect(status_effect.id)
                            {
                                if health_points.hp > data.apply_per_second_value {
                                    health_points.hp -= data.apply_per_second_value;
                                } else {
                                    // Apply as damage so the entity dies
                                    damage_events.send(DamageEvent::with_attack(
                                        entity,
                                        entity,
                                        Damage {
                                            amount: data.apply_per_second_value as u32,
                                            is_critical: false,
                                            apply_hit_stun: false,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }

                // Check if expire time has been reached
                if let Some(expire_time) = status_effects.expire_times[status_effect_type] {
                    if expire_time <= server_time.now {
                        expired_status_effects[status_effect_type] = true;
                    }
                }
            }
        }

        // Check if any regen has expired
        for (status_effect_type, regen_slot) in status_effects_regen.regens.iter() {
            if let Some(regen) = regen_slot.as_ref() {
                if regen.applied_value == regen.total_value {
                    expired_status_effects[status_effect_type] = true;
                }
            }
        }

        if expired_status_effects.iter().any(|(_, expired)| *expired) {
            // Remove expired status effects
            let mut cleared_hp = false;
            let mut cleared_mp = false;

            for expired_status_effect_type in
                expired_status_effects
                    .iter()
                    .filter_map(|(status_effect_type, expired)| {
                        if *expired {
                            Some(status_effect_type)
                        } else {
                            None
                        }
                    })
            {
                status_effects.active[expired_status_effect_type] = None;
                status_effects.expire_times[expired_status_effect_type] = None;
                status_effects_regen.regens[expired_status_effect_type] = None;

                match expired_status_effect_type {
                    StatusEffectType::IncreaseHp => cleared_hp = true,
                    StatusEffectType::IncreaseMp => cleared_mp = true,
                    _ => {}
                }
            }

            // Update ability values adjust
            let mut ability_values = ability_values.clone();
            ability_values.adjust = (&*status_effects).into();

            // Immediately adjust hp / mp for the update packet
            let max_hp = ability_values.get_max_health();
            let max_mp = ability_values.get_max_mana();

            if health_points.hp > max_hp {
                health_points.hp = max_hp;
            }

            // Avoid borrowing mana_points as mutable if possible to prevent unnecessary change detection
            let mp_requires_limiting = mana_points
                .as_ref()
                .map(|mp| mp.mp > max_mp)
                .unwrap_or(false);
            if mp_requires_limiting {
                if let Some(mana_points) = mana_points.as_mut() {
                    mana_points.mp = max_mp;
                }
            }

            // Send status effect expiry message
            let mut updated_values = Vec::new();
            if cleared_hp {
                updated_values.push(health_points.hp);
            }

            if cleared_mp {
                updated_values.push(mana_points.as_ref().map(|mp| mp.mp).unwrap_or(0));
            }

            server_messages.send_entity_message(
                client_entity,
                ServerMessage::UpdateStatusEffects(UpdateStatusEffects {
                    entity_id: client_entity.id,
                    status_effects: status_effects.active.clone(),
                    updated_values,
                }),
            );
        }
    }
}
