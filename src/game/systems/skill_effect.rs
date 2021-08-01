use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use log::warn;

use crate::{
    data::{SkillAddAbility, SkillData, SkillType, StatusEffectType},
    game::{
        components::{AbilityValues, ClientEntity, Position, StatusEffects},
        messages::server::{ApplySkillEffect, ServerMessage},
        resources::{
            ClientEntityList, PendingSkillEffect, PendingSkillEffectList, PendingSkillEffectTarget,
            ServerMessages, ServerTime,
        },
        GameData,
    },
};

enum SkillCastError {
    InvalidSkill,
    InvalidTarget,
    NotEnoughUseAbility,
}

struct SkillWorld<'a> {
    client_entity_list: &'a ClientEntityList,
    game_data: &'a GameData,
    server_messages: &'a mut ServerMessages,
}

struct SkillCaster<'a> {
    entity: &'a Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
}

struct SkillTargetEntity<'a> {
    entity: &'a Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    status_effects: &'a mut StatusEffects,
}

fn apply_skill_status_effects_to_entity(
    skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetEntity,
    skill_data: &SkillData,
) -> Result<(), SkillCastError> {
    // TODO: Check skill_data.target_filter

    if skill_data.harm > 0 {
        // Apply damage to target
    }

    // TODO: Apply skill status
    for add_ability in skill_data.add_ability.iter() {
        match *add_ability {
            SkillAddAbility::Value(ability_type, value) => {
                match ability_type {
                    /*
                    TODO:
                    AbilityType::Stamina => {},
                    AbilityType::Money => {},
                    AbilityType::Health => {},
                    AbilityType::Mana => {},
                    */
                    _ => warn!(
                        "Unimplemented skill status effect add ability_type {:?}, value {}",
                        ability_type, value
                    ),
                }
            }
            _ => {}
        }
    }

    let mut effect_success = [false, false];
    for (effect_index, status_effect_id) in skill_data.status_effects.iter().enumerate() {
        if let Some(status_effect_data) = skill_world
            .game_data
            .status_effects
            .get_status_effect(*status_effect_id)
        {
            if skill_data.success_ratio > 0 {
                // TODO: Check success chance
            }

            // TODO: Compute value
            let value = 100;

            if skill_target
                .status_effects
                .can_apply(status_effect_data, value)
            {
                skill_target.status_effects.apply_status_effect(
                    status_effect_data,
                    skill_data.status_effect_duration,
                    value,
                );

                match status_effect_data.status_effect_type {
                    StatusEffectType::Fainting | StatusEffectType::Sleep => {
                        // TODO: Set current + next command to stop
                    }
                    StatusEffectType::Taunt => {
                        // TODO: Set current + next command to attack spell cast entity
                    }
                    _ => {}
                }

                effect_success[effect_index] = true;
            }
        }
    }

    // TODO: Update ability values, move speed, hp / mp
    if effect_success.iter().any(|x| *x) {
        skill_world.server_messages.send_entity_message(
            skill_target.client_entity,
            ServerMessage::ApplySkillEffect(ApplySkillEffect {
                entity_id: skill_target.client_entity.id,
                caster_entity_id: skill_caster.client_entity.id,
                caster_intelligence: skill_caster.ability_values.intelligence as i32,
                skill_id: skill_data.id,
                effect_success,
            }),
        );
    }

    Ok(())
}

fn apply_skill_status_effects(
    skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &PendingSkillEffectTarget,
    skill_data: &SkillData,
    target_query: &mut Query<(&ClientEntity, &Position, &mut StatusEffects)>,
    target_query_world: &mut SubWorld,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = skill_world
            .client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match skill_target {
            PendingSkillEffectTarget::Entity(target_entity) => {
                if let Ok((_, target_position, _)) =
                    target_query.get_mut(target_query_world, *target_entity)
                {
                    Some(target_position.position.xy())
                } else {
                    None
                }
            }
            PendingSkillEffectTarget::Position(position) => Some(*position),
        }
        .ok_or(SkillCastError::InvalidTarget)?;

        for (target_entity, _) in client_entity_zone
            .iter_entities_within_distance(skill_position, skill_data.scope as f32)
        {
            if let Ok((target_client_entity, target_position, target_status_effects)) =
                target_query.get_mut(target_query_world, target_entity)
            {
                apply_skill_status_effects_to_entity(
                    skill_world,
                    skill_caster,
                    &mut SkillTargetEntity {
                        entity: &target_entity,
                        client_entity: target_client_entity,
                        position: target_position,
                        status_effects: target_status_effects,
                    },
                    skill_data,
                )
                .ok();
            }
        }

        Ok(())
    } else if let PendingSkillEffectTarget::Entity(target_entity) = skill_target {
        if let Ok((target_client_entity, target_position, target_status_effects)) =
            target_query.get_mut(target_query_world, *target_entity)
        {
            // Apply only to target entity
            apply_skill_status_effects_to_entity(
                skill_world,
                skill_caster,
                &mut SkillTargetEntity {
                    entity: &target_entity,
                    client_entity: target_client_entity,
                    position: target_position,
                    status_effects: target_status_effects,
                },
                skill_data,
            )
        } else {
            Err(SkillCastError::InvalidTarget)
        }
    } else {
        Err(SkillCastError::InvalidTarget)
    }
}

#[allow(clippy::type_complexity)]
#[system]
pub fn skill_effect(
    world: &mut SubWorld,
    cmd: &mut CommandBuffer,
    caster_query: &mut Query<(&ClientEntity, &Position, &AbilityValues)>,
    target_query: &mut Query<(&ClientEntity, &Position, &mut StatusEffects)>,
    #[resource] game_data: &GameData,
    #[resource] client_entity_list: &ClientEntityList,
    #[resource] pending_skill_effect_list: &mut PendingSkillEffectList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] server_time: &ServerTime,
) {
    let (mut target_query_world, world) = world.split_for_query(&target_query);
    let caster_world = world;

    let mut skill_world = SkillWorld {
        client_entity_list,
        game_data,
        server_messages,
    };

    // TODO: drain_filter pls
    let mut i = 0;
    while i != pending_skill_effect_list.len() {
        if pending_skill_effect_list[i].when > server_time.now {
            i += 1;
            continue;
        }

        let PendingSkillEffect {
            skill_id,
            caster_entity,
            skill_target,
            ..
        } = pending_skill_effect_list.remove(i);

        let skill_data = skill_world.game_data.skills.get_skill(skill_id);
        if skill_data.is_none() {
            continue;
        }
        let skill_data = skill_data.unwrap();

        if let Ok((caster_client_entity, caster_position, caster_ability_values)) =
            caster_query.get(&caster_world, caster_entity)
        {
            let skill_caster = SkillCaster {
                entity: &caster_entity,
                client_entity: caster_client_entity,
                position: caster_position,
                ability_values: caster_ability_values,
            };

            let result = match skill_data.skill_type {
                SkillType::SelfBoundDuration
                | SkillType::SelfStateDuration
                | SkillType::TargetBoundDuration
                | SkillType::TargetStateDuration => apply_skill_status_effects(
                    &mut skill_world,
                    &skill_caster,
                    &skill_target,
                    skill_data,
                    target_query,
                    &mut target_query_world,
                ),
                _ => {
                    warn!("Unimplemented skill used {:?}", skill_data);
                    Ok(())
                }
            };

            match result {
                Ok(_) => skill_world.server_messages.send_entity_message(
                    caster_client_entity,
                    ServerMessage::FinishCastingSkill(caster_client_entity.id, skill_id),
                ),
                Err(_) => {
                    // TODO: Send skill cast cancel
                }
            }
        }
    }
}
