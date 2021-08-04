use bevy_ecs::prelude::{Query, Res, ResMut};

use crate::{
    data::StatusEffectType,
    game::{
        components::{AbilityValues, ClientEntity, HealthPoints, ManaPoints, StatusEffects},
        messages::server::{ServerMessage, UpdateStatusEffects},
        resources::{ServerMessages, ServerTime},
    },
};

pub fn status_effect_system(
    mut query: Query<(
        &ClientEntity,
        &AbilityValues,
        &mut HealthPoints,
        &mut ManaPoints,
        &mut StatusEffects,
    )>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    for (client_entity, ability_values, mut health_points, mut mana_points, mut status_effects) in
        query.iter_mut()
    {
        let mut status_effects_expired = false;

        for (_, status_effect_slot) in status_effects.active.iter() {
            if let Some(status_effect) = status_effect_slot {
                // TODO: Process per tick status effect: IncreaseHp, IncreaseMp, Poisoned, DecreaseLifeTime

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
                    if status_effect.expire_time <= server_time.now {
                        *status_effect_slot = None;

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
            let max_hp = ability_values.get_max_health() as u32;
            let max_mp = ability_values.get_max_health() as u32;

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
