use bevy_ecs::prelude::{Entity, EventReader, EventWriter, Local, Mut, Query, Res, ResMut};
use log::warn;
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::{
        Damage, SkillData, SkillTargetFilter, SkillType, StatusEffectClearedByType,
        StatusEffectType,
    },
    game::{
        bundles::ability_values_get_value,
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityType, Equipment,
            HealthPoints, Level, MoveSpeed, Npc, Position, SkillList, StatusEffects, Team,
        },
        events::{DamageEvent, SkillEvent, SkillEventTarget},
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

struct SkillWorld<'a, 'b, 'c, 'd, 'e, 'f> {
    client_entity_list: &'a ClientEntityList,
    game_data: &'b GameData,
    server_messages: &'c mut ResMut<'d, ServerMessages>,
    server_time: &'b ServerTime,
    damage_events: &'e mut EventWriter<'f, DamageEvent>,
    rng: ThreadRng,
}

struct SkillCaster<'a> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    level: &'a Level,
    team: &'a Team,
}

#[allow(dead_code)]
struct SkillTargetData<'a> {
    entity: Entity,
    client_entity: &'a ClientEntity,
    position: &'a Position,
    ability_values: &'a AbilityValues,
    status_effects: Mut<'a, StatusEffects>,
    team: &'a Team,
    health_points: &'a HealthPoints,
    level: &'a Level,
    move_speed: &'a MoveSpeed,

    // To update character ability_values
    character_info: Option<&'a CharacterInfo>,
    equipment: Option<&'a Equipment>,
    basic_stats: Option<&'a BasicStats>,
    skill_list: Option<&'a SkillList>,

    // To update NPC ability_values
    npc: Option<&'a Npc>,
}

#[allow(clippy::type_complexity)]
struct SkillTargetQuery<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k, 'l, 'm, 'n> {
    query: Query<
        'a,
        (
            &'b ClientEntity,
            &'c Position,
            &'d AbilityValues,
            &'e mut StatusEffects,
            &'f Team,
            &'g HealthPoints,
            &'h Level,
            &'i MoveSpeed,
            Option<&'j CharacterInfo>,
            Option<&'k Equipment>,
            Option<&'l BasicStats>,
            Option<&'m SkillList>,
            Option<&'n Npc>,
        ),
    >,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k, 'l, 'm, 'n>
    SkillTargetQuery<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, 'j, 'k, 'l, 'm, 'n>
{
    fn get_skill_target_data(&mut self, entity: Entity) -> Option<SkillTargetData> {
        let (
            client_entity,
            position,
            ability_values,
            status_effects,
            team,
            health_points,
            level,
            move_speed,
            character_info,
            equipment,
            basic_stats,
            skill_list,
            npc,
        ) = self.query.get_mut(entity).ok()?;

        Some(SkillTargetData {
            entity,
            client_entity,
            position,
            ability_values,
            status_effects,
            team,
            health_points,
            level,
            move_speed,
            character_info,
            equipment,
            basic_stats,
            skill_list,
            npc,
        })
    }
}

fn check_skill_target_filter(
    _skill_world: &mut SkillWorld,
    skill_caster: &SkillCaster,
    skill_target: &mut SkillTargetData,
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
    skill_target: &mut SkillTargetData,
    skill_data: &SkillData,
) -> Result<(), SkillCastError> {
    if !check_skill_target_filter(skill_world, skill_caster, skill_target, skill_data) {
        return Err(SkillCastError::InvalidTarget);
    }

    if skill_data.harm != 0 {
        skill_world.damage_events.send(DamageEvent::new(
            skill_caster.entity,
            skill_target.entity,
            Damage {
                amount: 0,
                is_critical: false,
                apply_hit_stun: false,
            },
        ));
    }

    for add_ability in skill_data.add_ability.iter() {
        match add_ability.ability_type {
            /*
            TODO:
            AbilityType::Stamina => {},
            AbilityType::Money => {},
            AbilityType::Health => {},
            AbilityType::Mana => {},
            */
            _ => warn!(
                "Unimplemented skill status effect add ability_type {:?}, value {}",
                add_ability.ability_type, add_ability.value
            ),
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
                match status_effect_data.cleared_by_type {
                    StatusEffectClearedByType::ClearGood => {
                        if skill_data.success_ratio
                            < skill_target.level.level as i32 - skill_caster.level.level as i32
                                + skill_world.rng.gen_range(1..=100)
                        {
                            continue;
                        }
                    }
                    _ => {
                        if skill_data.success_ratio as f32
                            * (skill_caster.level.level as i32 * 2
                                + skill_caster.ability_values.get_intelligence()
                                + 20) as f32
                            / (skill_target.ability_values.get_resistance() as f32 * 0.6
                                + 5.0
                                + skill_target.ability_values.get_avoid() as f32)
                            <= skill_world.rng.gen_range(1..=100) as f32
                        {
                            continue;
                        }
                    }
                }
            }

            let adjust_value =
                if let Some(skill_add_ability) = skill_data.add_ability.get(effect_index) {
                    let ability_value = ability_values_get_value(
                        skill_add_ability.ability_type,
                        skill_target.ability_values,
                        skill_target.level,
                        skill_target.move_speed,
                        skill_target.team,
                        skill_target.character_info,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    )
                    .unwrap_or(0);
                    skill_world
                        .game_data
                        .ability_value_calculator
                        .calculate_skill_adjust_value(
                            skill_add_ability,
                            skill_caster.ability_values.get_intelligence(),
                            ability_value,
                        )
                } else {
                    0
                };

            if skill_target
                .status_effects
                .can_apply(status_effect_data, adjust_value)
            {
                skill_target.status_effects.apply_status_effect(
                    status_effect_data,
                    skill_world.server_time.now + skill_data.status_effect_duration,
                    adjust_value,
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
    skill_target_query: &mut SkillTargetQuery,
) -> Result<(), SkillCastError> {
    if skill_data.scope > 0 {
        // Apply in AOE around target position
        let client_entity_zone = skill_world
            .client_entity_list
            .get_zone(skill_caster.position.zone_id)
            .ok_or(SkillCastError::InvalidTarget)?;

        let skill_position = match *skill_target {
            SkillEventTarget::Entity(target_entity) => {
                if let Some(skill_target) = skill_target_query.get_skill_target_data(target_entity)
                {
                    Some(skill_target.position.position.xy())
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
            if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity)
            {
                apply_skill_status_effects_to_entity(
                    skill_world,
                    skill_caster,
                    &mut skill_target,
                    skill_data,
                )
                .ok();
            }
        }

        Ok(())
    } else if let SkillEventTarget::Entity(target_entity) = *skill_target {
        if let Some(mut skill_target) = skill_target_query.get_skill_target_data(target_entity) {
            apply_skill_status_effects_to_entity(
                skill_world,
                skill_caster,
                &mut skill_target,
                skill_data,
            )
            .ok();
            Ok(())
        } else {
            Err(SkillCastError::InvalidTarget)
        }
    } else {
        Err(SkillCastError::InvalidTarget)
    }
}

#[allow(clippy::type_complexity)]
pub fn skill_effect_system(
    caster_query: Query<(&ClientEntity, &Position, &AbilityValues, &Team, &Level)>,
    target_query: Query<(
        &ClientEntity,
        &Position,
        &AbilityValues,
        &mut StatusEffects,
        &Team,
        &HealthPoints,
        &Level,
        &MoveSpeed,
        Option<&CharacterInfo>,
        Option<&Equipment>,
        Option<&BasicStats>,
        Option<&SkillList>,
        Option<&Npc>,
    )>,
    game_data: Res<GameData>,
    client_entity_list: Res<ClientEntityList>,
    mut skill_events: EventReader<SkillEvent>,
    mut damage_events: EventWriter<DamageEvent>,
    mut pending_skill_events: Local<Vec<SkillEvent>>,
    mut server_messages: ResMut<ServerMessages>,
    server_time: Res<ServerTime>,
) {
    let mut skill_world = SkillWorld {
        client_entity_list: &client_entity_list,
        damage_events: &mut damage_events,
        game_data: &game_data,
        server_messages: &mut server_messages,
        server_time: &server_time,
        rng: rand::thread_rng(),
    };
    let mut skill_target_query = SkillTargetQuery {
        query: target_query,
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

        if let Ok((
            caster_client_entity,
            caster_position,
            caster_ability_values,
            caster_team,
            caster_level,
        )) = caster_query.get(caster_entity)
        {
            let skill_caster = SkillCaster {
                entity: caster_entity,
                client_entity: caster_client_entity,
                level: caster_level,
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
                    &mut skill_target_query,
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
