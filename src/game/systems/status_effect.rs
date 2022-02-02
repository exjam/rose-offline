use bevy_ecs::prelude::{Query, Res, ResMut};

use crate::{
    data::StatusEffectType,
    game::{
        components::{
            AbilityValues, ActiveStatusEffectRegen, ClientEntity, HealthPoints, ManaPoints,
            StatusEffects, StatusEffectsRegen,
        },
        messages::server::{ServerMessage, UpdateStatusEffects},
        resources::{ServerMessages, ServerTime},
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
        &ClientEntity,
        &AbilityValues,
        &mut HealthPoints,
        &mut ManaPoints,
        &mut StatusEffects,
        &mut StatusEffectsRegen,
    )>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    for (
        client_entity,
        ability_values,
        mut health_points,
        mut mana_points,
        mut status_effects,
        mut status_effects_regen,
    ) in query.iter_mut()
    {
        let mut status_effects_expired = false;

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
                            // Calculate regen for this tick
                            let regen =
                                update_status_effect_regen(status_effect_regen, &server_time);

                            // Update mp
                            let max_mp = ability_values.get_max_mana();
                            mana_points.mp = i32::min(mana_points.mp + regen, max_mp);

                            // Expire status effect if hp has reached max
                            if mana_points.mp == max_mp {
                                status_effect_regen.applied_value = status_effect_regen.total_value;
                                status_effects_expired = true;
                            }
                        }
                    }
                    StatusEffectType::Poisoned => {}
                    StatusEffectType::DecreaseLifeTime => {}
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

            if mana_points.mp > max_mp {
                mana_points.mp = max_mp;
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
