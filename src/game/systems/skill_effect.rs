use bevy_ecs::prelude::{Entity, EventReader, Local, Mut, Query, Res, ResMut};
use log::warn;

use crate::{
    data::{SkillAddAbility, SkillData, SkillTargetFilter, SkillType, StatusEffectType},
    game::{
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment,
            HealthPoints, Level, MoveMode, Npc, Position, SkillList, StatusEffects, Team,
        },
        events::{SkillEvent, SkillEventTarget},
        messages::server::{ApplySkillEffect, CancelCastingSkillReason, ServerMessage},
        resources::{ClientEntityList, ServerMessages, ServerTime},
        GameData,
    },
};

#[allow(dead_code)]
enum SkillCastError {
    InvalidSkill,
    InvalidTarget,
    NotEnoughUseAbility,
}

struct SkillWorld<'c, 'd, 'e, 'f> {
    client_entity_list: &'c ClientEntityList,
    game_data: &'d GameData,
    server_messages: &'e mut ResMut<'f, ServerMessages>,
    server_time: &'d ServerTime,
}

struct SkillCaster<'a> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    team: &'a Team,
}

#[allow(dead_code)]
struct SkillTargetEntity<'a, 'b> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    status_effects: &'a mut Mut<'b, StatusEffects>,
    team: &'a Team,
    health_points: &'a HealthPoints,
    level: &'a Level,
    move_mode: &'a MoveMode,

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
                    skill_world.server_time.now + skill_data.status_effect_duration,
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

    if effect_success.iter().any(|x| *x) {
        skill_world.server_messages.send_entity_message(
            skill_target.client_entity,
            ServerMessage::ApplySkillEffect(ApplySkillEffect {
                entity_id: skill_target.client_entity.id,
                caster_entity_id: skill_caster.client_entity.id,
                caster_intelligence: skill_caster.ability_values.get_intelligence(),
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
    skill_target: &SkillEventTarget,
    skill_data: &SkillData,
    target_query: &mut Query<(
        &ClientEntity,
        &Position,
        &AbilityValues,
        &mut StatusEffects,
        &Team,
        &HealthPoints,
        &Level,
        &MoveMode,
        Option<&CharacterInfo>,
        Option<&Equipment>,
        Option<&BasicStats>,
        Option<&SkillList>,
        Option<&Npc>,
    )>,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = skill_world
            .client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Ok((_, target_position, ..)) = target_query.get_mut(target_entity) {
                    Some(target_position.position.xy())
                } else {
                    None
                }
            }
            SkillEventTarget::Position(position) => Some(position),
        }
        .ok_or(SkillCastError::InvalidTarget)?;

        for (target_entity, _) in client_entity_zone
            .iter_entities_within_distance(skill_position, skill_data.scope as f32)
        {
            if let Ok((
                target_client_entity,
                target_position,
                target_ability_values,
                mut target_status_effects,
                target_team,
                target_health_points,
                target_level,
                target_move_mode,
                target_character_info,
                target_equipment,
                target_basic_stats,
                target_skill_list,
                target_npc,
            )) = target_query.get_mut(target_entity)
            {
                apply_skill_status_effects_to_entity(
                    skill_world,
                    skill_caster,
                    &mut SkillTargetEntity {
                        entity: target_entity,
                        client_entity: target_client_entity,
                        position: target_position,
                        ability_values: target_ability_values,
                        status_effects: &mut target_status_effects,
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
    } else if let SkillEventTarget::Entity(target_entity) = *skill_target {
        if let Ok((
            target_client_entity,
            target_position,
            target_ability_values,
            mut target_status_effects,
            target_team,
            target_health_points,
            target_level,
            target_move_mode,
            target_character_info,
            target_equipment,
            target_basic_stats,
            target_skill_list,
            target_npc,
        )) = target_query.get_mut(target_entity)
        {
            // Apply only to target entity
            apply_skill_status_effects_to_entity(
                skill_world,
                skill_caster,
                &mut SkillTargetEntity {
                    entity: target_entity,
                    client_entity: target_client_entity,
                    position: target_position,
                    ability_values: target_ability_values,
                    status_effects: &mut target_status_effects,
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
pub fn skill_effect_system(
    caster_query: Query<(&ClientEntity, &Position, &AbilityValues, &Team)>,
    mut target_query: Query<(
        &ClientEntity,
        &Position,
        &AbilityValues,
        &mut StatusEffects,
        &Team,
        &HealthPoints,
        &Level,
        &MoveMode,
        Option<&CharacterInfo>,
        Option<&Equipment>,
        Option<&BasicStats>,
        Option<&SkillList>,
        Option<&Npc>,
    )>,
    game_data: Res<GameData>,
    client_entity_list: Res<ClientEntityList>,
    mut skill_events: EventReader<SkillEvent>,
    mut pending_skill_events: Local<Vec<SkillEvent>>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    let mut skill_world = SkillWorld {
        client_entity_list: &client_entity_list,
        game_data: &game_data,
        server_messages: &mut server_messages,
        server_time: &server_time,
    };

    // Read events into pending_skill_events for executing at specific time
    for skill_event in skill_events.iter() {
        pending_skill_events.push(skill_event.clone());
    }

    // TODO: drain_filter pls
    let mut i = 0;
    while i != pending_skill_events.len() {
        if pending_skill_events[i].when > server_time.now {
            i += 1;
            continue;
        }

        let SkillEvent {
            skill_id,
            caster_entity,
            skill_target,
            ..
        } = pending_skill_events.remove(i);

        let skill_data = skill_world.game_data.skills.get_skill(skill_id);
        if skill_data.is_none() {
            continue;
        }
        let skill_data = skill_data.unwrap();

        if let Ok((caster_client_entity, caster_position, caster_ability_values, caster_team)) =
            caster_query.get(caster_entity)
        {
            let skill_caster = SkillCaster {
                entity: caster_entity,
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
                    &mut target_query,
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
