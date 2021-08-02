use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use log::warn;

use crate::{
    data::{SkillAddAbility, SkillData, SkillTargetFilter, SkillType, StatusEffectType},
    game::{
        bundles::client_entity_recalculate_ability_values,
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment,
            HealthPoints, Level, MoveMode, Npc, Position, SkillList, StatusEffects, Team,
        },
        messages::server::{ApplySkillEffect, CancelCastingSkillReason, ServerMessage},
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
    cmd: &'a mut CommandBuffer,
    client_entity_list: &'a ClientEntityList,
    game_data: &'a GameData,
    server_messages: &'a mut ServerMessages,
}

struct SkillCaster<'a> {
    entity: &'a Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    team: &'a Team,
}

struct SkillTargetEntity<'a> {
    entity: &'a Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    status_effects: &'a mut StatusEffects,
    team: &'a Team,
    health_points: &'a HealthPoints,
    level: &'a Level,
    move_mode: Option<&'a MoveMode>,

    // To update character ability_values
    character_info: Option<&'a CharacterInfo>,
    equipment: Option<&'a Equipment>,
    basic_stats: Option<&'a BasicStats>,
    skill_list: Option<&'a SkillList>,

    // To update NPC ability_values
    npc: Option<&'a Npc>,
}

fn check_skill_target_filter(
    _skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetEntity,
    skill_data: &SkillData,
) -> bool {
    match skill_data.target_filter {
        SkillTargetFilter::OnlySelf => skill_caster.entity == skill_target.entity,
        SkillTargetFilter::Group => true, // TODO: Implement SkillTargetFilter::Group
        SkillTargetFilter::Guild => true, // TODO: Implement SkillTargetFilter::Guild
        SkillTargetFilter::Allied => skill_caster.team.id == skill_target.team.id,
        SkillTargetFilter::Monster => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Monster
        ),
        SkillTargetFilter::Enemy => skill_caster.team.id != skill_target.team.id,
        SkillTargetFilter::EnemyCharacter => {
            skill_caster.team.id != skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::Character => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Character
        ),
        SkillTargetFilter::CharacterOrMonster => matches!(
            skill_target.client_entity.entity_type,
            ClientEntityType::Character | ClientEntityType::Monster
        ),
        SkillTargetFilter::DeadAlliedCharacter => {
            skill_caster.team.id == skill_target.team.id
                && skill_target.health_points.hp == 0
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Character
                )
        }
        SkillTargetFilter::EnemyMonster => {
            skill_caster.team.id != skill_target.team.id
                && matches!(
                    skill_target.client_entity.entity_type,
                    ClientEntityType::Monster
                )
        }
    }
}

fn apply_skill_status_effects_to_entity(
    skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetEntity,
    skill_data: &SkillData,
) -> Result<(), SkillCastError> {
    if !check_skill_target_filter(skill_world, skill_caster, skill_target, skill_data) {
        return Err(SkillCastError::InvalidTarget);
    }

    if skill_data.harm > 0 {
        // TODO: Apply damage to target
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
    for (effect_index, status_effect_id) in skill_data
        .status_effects
        .iter()
        .enumerate()
        .filter_map(|(index, id)| id.map(|id| (index, id)))
    {
        if let Some(status_effect_data) = skill_world
            .game_data
            .status_effects
            .get_status_effect(status_effect_id)
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

    // Update ability values
    client_entity_recalculate_ability_values(
        skill_world.cmd,
        skill_world.game_data.ability_value_calculator.as_ref(),
        skill_target.client_entity,
        skill_target.entity,
        skill_target.status_effects,
        skill_target.basic_stats,
        skill_target.character_info,
        skill_target.equipment,
        Some(skill_target.level),
        skill_target.move_mode,
        skill_target.skill_list,
        skill_target.npc,
        None, // TODO: Update of skill target HP / MP
        None,
    );

    if effect_success.iter().any(|x| *x) {
        skill_world.server_messages.send_entity_message(
            skill_target.client_entity,
            ServerMessage::ApplySkillEffect(ApplySkillEffect {
                entity_id: skill_target.client_entity.id,
                caster_entity_id: skill_caster.client_entity.id,
                caster_intelligence: skill_caster.ability_values.intelligence,
                skill_id: skill_data.id,
                effect_success,
            }),
        );
    }

    Ok(())
}

#[allow(clippy::type_complexity)]
fn apply_skill_status_effects(
    skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &PendingSkillEffectTarget,
    skill_data: &SkillData,
    target_query: &mut Query<(
        &ClientEntity,
        &Position,
        &mut StatusEffects,
        &Team,
        &HealthPoints,
        &Level,
        Option<&MoveMode>,
        Option<&CharacterInfo>,
        Option<&Equipment>,
        Option<&BasicStats>,
        Option<&SkillList>,
        Option<&Npc>,
    )>,
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
                if let Ok((_, target_position, ..)) =
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
            if let Ok((
                target_client_entity,
                target_position,
                target_status_effects,
                target_team,
                target_health_points,
                target_level,
                target_move_mode,
                target_character_info,
                target_equipment,
                target_basic_stats,
                target_skill_list,
                target_npc,
            )) = target_query.get_mut(target_query_world, target_entity)
            {
                apply_skill_status_effects_to_entity(
                    skill_world,
                    skill_caster,
                    &mut SkillTargetEntity {
                        entity: &target_entity,
                        client_entity: target_client_entity,
                        position: target_position,
                        status_effects: target_status_effects,
                        team: target_team,
                        health_points: target_health_points,
                        level: target_level,
                        move_mode: target_move_mode,
                        character_info: target_character_info,
                        equipment: target_equipment,
                        basic_stats: target_basic_stats,
                        skill_list: target_skill_list,
                        npc: target_npc,
                    },
                    skill_data,
                )
                .ok();
            }
        }

        Ok(())
    } else if let PendingSkillEffectTarget::Entity(target_entity) = skill_target {
        if let Ok((
            target_client_entity,
            target_position,
            target_status_effects,
            target_team,
            target_health_points,
            target_level,
            target_move_mode,
            target_character_info,
            target_equipment,
            target_basic_stats,
            target_skill_list,
            target_npc,
        )) = target_query.get_mut(target_query_world, *target_entity)
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
                    team: target_team,
                    health_points: target_health_points,
                    level: target_level,
                    move_mode: target_move_mode,
                    character_info: target_character_info,
                    equipment: target_equipment,
                    basic_stats: target_basic_stats,
                    skill_list: target_skill_list,
                    npc: target_npc,
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
    caster_query: &mut Query<(&ClientEntity, &Position, &AbilityValues, &Team)>,
    target_query: &mut Query<(
        &ClientEntity,
        &Position,
        &mut StatusEffects,
        &Team,
        &HealthPoints,
        &Level,
        Option<&MoveMode>,
        Option<&CharacterInfo>,
        Option<&Equipment>,
        Option<&BasicStats>,
        Option<&SkillList>,
        Option<&Npc>,
    )>,
    #[resource] game_data: &GameData,
    #[resource] client_entity_list: &ClientEntityList,
    #[resource] pending_skill_effect_list: &mut PendingSkillEffectList,
    #[resource] server_messages: &mut ServerMessages,
    #[resource] server_time: &ServerTime,
) {
    let (mut target_query_world, world) = world.split_for_query(&target_query);
    let caster_world = world;

    let mut skill_world = SkillWorld {
        cmd,
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

        if let Ok((caster_client_entity, caster_position, caster_ability_values, caster_team)) =
            caster_query.get(&caster_world, caster_entity)
        {
            let skill_caster = SkillCaster {
                entity: &caster_entity,
                client_entity: caster_client_entity,
                position: caster_position,
                ability_values: caster_ability_values,
                team: caster_team,
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
                Err(error) => skill_world.server_messages.send_entity_message(
                    caster_client_entity,
                    ServerMessage::CancelCastingSkill(
                        caster_client_entity.id,
                        match error {
                            SkillCastError::NotEnoughUseAbility => {
                                CancelCastingSkillReason::NeedAbility
                            }
                            _ => CancelCastingSkillReason::NeedTarget,
                        },
                    ),
                ),
            }
        }
    }
}
