use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    prelude::{Query, Res, ResMut},
};
use std::time::Duration;

use rose_data::StatusEffectType;

use crate::{
    data::Damage,
    game::{
        components::{
            AbilityValues, ActiveStatusEffectRegen, ClientEntity, HealthPoints, ManaPoints,
            StatusEffects, StatusEffectsRegen,
        },
        events::DamageEvent,
        messages::server::{ServerMessage, UpdateStatusEffects},
        resources::{ServerMessages, ServerTime},
        GameData,
    },
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
        let mut status_effects_expired = false;
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

                            // Expire status effect if hp has reached max
                            if health_points.hp == max_hp {
                                status_effect_regen.applied_value = status_effect_regen.total_value;
                                status_effects_expired = true;
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

                                // Expire status effect if hp has reached max
                                if mana_points.mp == max_mp {
                                    status_effect_regen.applied_value =
                                        status_effect_regen.total_value;
                                    status_effects_expired = true;
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

                if status_effect.expire_time <= server_time.now {
                    status_effects_expired = true;
                }
            }
        }

        if status_effects_expired {
            // Remove expired status effects
            let mut cleared_hp = false;
            let mut cleared_mp = false;

            for (status_effect_type, status_effect_slot) in status_effects.active.iter_mut() {
                if let Some(status_effect) = status_effect_slot {
                    let regen_slot = &mut status_effects_regen.regens[status_effect_type];
                    let regen_expired = regen_slot
                        .as_ref()
                        .map(|regen| regen.applied_value == regen.total_value)
                        .unwrap_or(false);

                    if status_effect.expire_time <= server_time.now || regen_expired {
                        *status_effect_slot = None;
                        *regen_slot = None;

                        match status_effect_type {
                            StatusEffectType::IncreaseHp | StatusEffectType::IncreaseMaxHp => {
                                cleared_hp = true
                            }
                            StatusEffectType::IncreaseMp | StatusEffectType::IncreaseMaxMp => {
                                cleared_mp = true
                            }
                            _ => {}
                        }
                    }
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
            let updated_hp = if cleared_hp {
                Some(*health_points)
            } else {
                None
            };

            let updated_mp = if cleared_mp {
                Some(
                    mana_points
                        .as_ref()
                        .map(|mp| *mp.as_ref())
                        .unwrap_or_else(|| ManaPoints::new(0)),
                )
            } else {
                None
            };

            server_messages.send_entity_message(
                client_entity,
                ServerMessage::UpdateStatusEffects(UpdateStatusEffects {
                    entity_id: client_entity.id,
                    status_effects: status_effects.clone(),
                    updated_hp,
                    updated_mp,
                }),
            );
        }
    }
}
