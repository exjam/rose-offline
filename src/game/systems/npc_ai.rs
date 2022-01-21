use std::{
    marker::PhantomData,
    num::NonZeroU8,
    ops::{Range, RangeInclusive},
    time::Duration,
};

use bevy_ecs::{
    prelude::{Commands, Entity, EventWriter, Query, Res, ResMut},
    system::SystemParam,
};
use chrono::{Datelike, Timelike};
use log::{trace, warn};
use nalgebra::{Point3, Vector3};
use rand::Rng;

use crate::{
    data::{
        formats::{
            AipAbilityType, AipAction, AipCondition, AipConditionFindNearbyEntities,
            AipConditionMonthDayTime, AipConditionWeekDayTime, AipDamageType, AipDistanceOrigin,
            AipEvent, AipHaveStatusTarget, AipHaveStatusType, AipMoveMode, AipMoveOrigin, AipNpcId,
            AipOperatorType, AipTrigger, AipVariableType,
        },
        Damage, NpcId,
    },
    game::{
        bundles::{client_entity_leave_zone, ItemDropBundle},
        components::{
            AbilityValues, ClientEntity, ClientEntitySector, Command, CommandData, CommandDie,
            DamageSources, GameClient, HealthPoints, Level, MonsterSpawnPoint, MoveMode,
            NextCommand, Npc, NpcAi, ObjectVariables, Owner, Position, SpawnOrigin, StatusEffects,
            Target, Team,
        },
        events::RewardXpEvent,
        messages::server::ServerMessage,
        resources::{ClientEntityList, ServerTime, WorldRates, WorldTime, ZoneList},
        GameData,
    },
};

const DAMAGE_REWARD_EXPIRE_TIME: Duration = Duration::from_secs(5 * 60);

#[derive(SystemParam)]
pub struct AiSystemParameters<'w, 's> {
    commands: Commands<'w, 's>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    target_query: Query<
        'w,
        's,
        (
            &'static Level,
            &'static Team,
            &'static AbilityValues,
            &'static StatusEffects,
            &'static HealthPoints,
        ),
    >,
    object_variable_query: Query<'w, 's, &'static mut ObjectVariables>,
    owner_query: Query<'w, 's, (&'static Position, Option<&'static Target>)>,
}

#[derive(SystemParam)]
pub struct AiSystemResources<'w, 's> {
    game_data: Res<'w, GameData>,
    server_time: Res<'w, ServerTime>,
    world_time: Res<'w, WorldTime>,
    zone_list: Res<'w, ZoneList>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

struct AiSourceData<'s> {
    entity: Entity,
    ability_values: &'s AbilityValues,
    health_points: &'s HealthPoints,
    level: &'s Level,
    owner: Option<Entity>,
    position: &'s Position,
    spawn_origin: Option<&'s SpawnOrigin>,
    status_effects: &'s StatusEffects,
    target: Option<Entity>,
    team: &'s Team,
}

struct AiAttackerData<'a> {
    entity: Entity,
    _position: &'a Position,
    _team: &'a Team,
    ability_values: &'a AbilityValues,
    health_points: &'a HealthPoints,
    level: &'a Level,
    // TODO: Missing data on if clan master
}

#[allow(dead_code)]
struct AiParameters<'a> {
    source: &'a AiSourceData<'a>,
    attacker: Option<&'a AiAttackerData<'a>>,
    find_char: Option<(Entity, Point3<f32>)>,
    near_char: Option<(Entity, Point3<f32>)>,
    damage_received: Option<Damage>,
    selected_local_npc: Option<Entity>,
    is_dead: bool,
}

enum AiConditionResult {
    Failed,
}

fn compare_aip_value(operator: AipOperatorType, value1: i32, value2: i32) -> bool {
    match operator {
        AipOperatorType::Equals => value1 == value2,
        AipOperatorType::GreaterThan => value1 > value2,
        AipOperatorType::GreaterThanEqual => value1 >= value2,
        AipOperatorType::LessThan => value1 < value2,
        AipOperatorType::LessThanEqual => value1 <= value2,
        AipOperatorType::NotEqual => value1 != value2,
    }
}

fn ai_condition_count_nearby_entities(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    distance: i32,
    is_allied: bool,
    level_diff_range: &RangeInclusive<i32>,
    count_operator_type: Option<AipOperatorType>,
    count: i32,
) -> Result<(), AiConditionResult> {
    let mut find_char = None;
    let mut near_char_distance = None;
    let mut find_count = 0;

    let zone_entities = ai_system_parameters
        .client_entity_list
        .get_zone(ai_parameters.source.position.zone_id)
        .ok_or(AiConditionResult::Failed)?;

    for (entity, position) in zone_entities
        .iter_entities_within_distance(ai_parameters.source.position.position.xy(), distance as f32)
    {
        // Ignore self entity
        if entity == ai_parameters.source.entity {
            continue;
        }

        // Check level and team requirements
        let meets_requirements =
            ai_system_parameters
                .target_query
                .get(entity)
                .map_or(false, |(level, team, ..)| {
                    let level_diff = ai_parameters.source.level.level as i32 - level.level as i32;

                    is_allied == (team.id == ai_parameters.source.team.id)
                        && level_diff_range.contains(&level_diff)
                });
        if !meets_requirements {
            continue;
        }

        // Update near char for nearest found character
        let distance_squared =
            (ai_parameters.source.position.position - position).magnitude_squared();
        if near_char_distance.map_or(true, |x| distance_squared < x) {
            ai_parameters.near_char = Some((entity, position));
            near_char_distance = Some(distance_squared);
        }

        // Continue until we have satisfy count
        find_count += 1;
        if count_operator_type.is_none() && find_count >= count {
            find_char = Some((entity, position));
            break;
        }
    }

    if let Some(operator) = count_operator_type {
        if compare_aip_value(operator, find_count, count) {
            find_char = ai_parameters.near_char;
        }
    }

    if find_char.is_some() {
        ai_parameters.find_char = find_char;
        Ok(())
    } else {
        Err(AiConditionResult::Failed)
    }
}

fn ai_condition_damage(
    ai_parameters: &mut AiParameters,
    damage_type: AipDamageType,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    match damage_type {
        AipDamageType::Given => false,
        AipDamageType::Received => compare_aip_value(
            operator,
            ai_parameters
                .damage_received
                .map_or(0, |damage| damage.amount as i32),
            value,
        ),
    }
}

fn ai_condition_distance(
    ai_system_parameters: &AiSystemParameters,
    ai_parameters: &mut AiParameters,
    origin: AipDistanceOrigin,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    let distance_squared = match origin {
        AipDistanceOrigin::Spawn => match ai_parameters.source.spawn_origin {
            Some(SpawnOrigin::MonsterSpawnPoint(_, spawn_position)) => Some(spawn_position.xy()),
            _ => None,
        },
        AipDistanceOrigin::Owner => ai_parameters
            .source
            .owner
            .and_then(|owner_entity| ai_system_parameters.owner_query.get(owner_entity).ok())
            .map(|(position, _)| position.position.xy()),
        AipDistanceOrigin::Target => ai_parameters
            .source
            .target
            .and_then(|target_entity| ai_system_parameters.owner_query.get(target_entity).ok())
            .map(|(position, _)| position.position.xy()),
    }
    .map(|compare_position| {
        (ai_parameters.source.position.position.xy() - compare_position).magnitude_squared() as i32
    });

    if let Some(distance_squared) = distance_squared {
        compare_aip_value(operator, distance_squared, value * value)
    } else {
        false
    }
}

fn ai_condition_health_percent(
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    value: i32,
) -> bool {
    let current = ai_parameters.source.health_points.hp as i32;
    let max = ai_parameters.source.ability_values.get_max_health();

    compare_aip_value(operator, (100 * current) / max, value)
}

fn ai_condition_has_no_owner(
    ai_system_parameters: &AiSystemParameters,
    ai_parameters: &mut AiParameters,
) -> bool {
    if let Some(owner_position) = ai_parameters
        .source
        .owner
        .and_then(|owner_entity| ai_system_parameters.owner_query.get(owner_entity).ok())
        .map(|(position, _)| position.clone())
    {
        // Our owner must be in the same map
        owner_position.zone_id != ai_parameters.source.position.zone_id
    } else {
        true
    }
}

fn ai_condition_is_attacker_current_target(ai_parameters: &mut AiParameters) -> bool {
    if let Some(attacker) = ai_parameters.attacker {
        if let Some(target) = ai_parameters.source.target {
            return attacker.entity == target;
        }
    }

    false
}

fn ai_condition_no_target_and_compare_attacker_ability_value(
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    ability: AipAbilityType,
    value: i32,
) -> bool {
    if ai_parameters.source.target.is_some() {
        return false;
    }

    if let Some(attacker) = ai_parameters.attacker {
        let ability_value = match ability {
            AipAbilityType::Level => attacker.level.level as i32,
            AipAbilityType::Attack => attacker.ability_values.get_attack_power(),
            AipAbilityType::Defence => attacker.ability_values.get_defence(),
            AipAbilityType::Resistance => attacker.ability_values.get_resistance(),
            AipAbilityType::HealthPoints => attacker.health_points.hp as i32,
            AipAbilityType::Charm => attacker.ability_values.get_charm(),
        };

        compare_aip_value(operator, ability_value, value)
    } else {
        false
    }
}

fn ai_condition_random(operator: AipOperatorType, range: Range<i32>, value: i32) -> bool {
    compare_aip_value(operator, rand::thread_rng().gen_range(range), value)
}

fn ai_condition_source_ability_value(
    ai_parameters: &mut AiParameters,
    operator: AipOperatorType,
    ability: AipAbilityType,
    value: i32,
) -> bool {
    let ability_value = match ability {
        AipAbilityType::Level => ai_parameters.source.level.level as i32,
        AipAbilityType::Attack => ai_parameters.source.ability_values.get_attack_power(),
        AipAbilityType::Defence => ai_parameters.source.ability_values.get_defence(),
        AipAbilityType::Resistance => ai_parameters.source.ability_values.get_resistance(),
        AipAbilityType::HealthPoints => ai_parameters.source.health_points.hp as i32,
        AipAbilityType::Charm => ai_parameters.source.ability_values.get_charm(),
    };

    compare_aip_value(operator, ability_value, value)
}

fn ai_condition_select_local_npc(
    ai_system_resources: &AiSystemResources,
    ai_parameters: &mut AiParameters,
    npc_id: AipNpcId,
) -> bool {
    let local_npc =
        NpcId::new(npc_id as u16).and_then(|npc_id| ai_system_resources.zone_list.find_npc(npc_id));
    ai_parameters.selected_local_npc = local_npc;
    local_npc.is_some()
}

fn ai_condition_month_day_time(
    ai_system_resources: &AiSystemResources,
    month_day: Option<NonZeroU8>,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &ai_system_resources.server_time.local_time;

    if let Some(month_day) = month_day {
        if month_day.get() as u32 != local_time.day() {
            return false;
        }
    }

    let local_day_minutes = local_time.hour() as i32 + local_time.minute() as i32;
    day_minutes_range.contains(&local_day_minutes)
}

fn ai_condition_week_day_time(
    ai_system_resources: &AiSystemResources,
    week_day: u8,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &ai_system_resources.server_time.local_time;

    if week_day as u32 != local_time.weekday().num_days_from_sunday() {
        return false;
    }

    let local_day_minutes = local_time.hour() as i32 + local_time.minute() as i32;
    day_minutes_range.contains(&local_day_minutes)
}

fn ai_condition_world_time(
    ai_system_resources: &AiSystemResources,
    range: &RangeInclusive<u32>,
) -> bool {
    range.contains(&ai_system_resources.world_time.ticks.get_world_time())
}

fn ai_condition_zone_time(
    ai_system_resources: &AiSystemResources,
    ai_parameters: &AiParameters,
    range: &RangeInclusive<u32>,
) -> bool {
    let world_time = ai_system_resources.world_time.ticks.get_world_time();
    let zone_time = if let Some(zone_data) = ai_system_resources
        .game_data
        .zones
        .get_zone(ai_parameters.source.position.zone_id)
    {
        world_time % zone_data.day_cycle
    } else {
        world_time
    };
    range.contains(&zone_time)
}

fn ai_condition_is_zone_daytime(
    ai_system_resources: &AiSystemResources,
    ai_parameters: &AiParameters,
    is_daytime: bool,
) -> bool {
    if let Some(zone_data) = ai_system_resources
        .game_data
        .zones
        .get_zone(ai_parameters.source.position.zone_id)
    {
        let world_time = ai_system_resources.world_time.ticks.get_world_time();
        let zone_time = world_time % zone_data.day_cycle;
        let zone_day_start = zone_data.day_time / 2;
        let zone_day_end = (zone_data.evening_time + zone_data.night_time) / 2;

        is_daytime == (zone_day_start..=zone_day_end).contains(&zone_time)
    } else {
        is_daytime
    }
}

fn ai_condition_variable(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &AiParameters,
    variable_type: AipVariableType,
    variable_id: usize,
    operator_type: AipOperatorType,
    value: i32,
) -> bool {
    let variable_value = match variable_type {
        AipVariableType::LocalNpcObject => ai_parameters
            .selected_local_npc
            .and_then(|object_entity| {
                ai_system_parameters
                    .object_variable_query
                    .get_mut(object_entity)
                    .ok()
            })
            .and_then(|object_variables| object_variables.variables.get(variable_id).copied())
            .unwrap_or(0),
        AipVariableType::Ai => ai_system_parameters
            .object_variable_query
            .get_mut(ai_parameters.source.entity)
            .ok()
            .and_then(|object_variables| object_variables.variables.get(variable_id).copied())
            .unwrap_or(0),
        AipVariableType::World => {
            warn!(
                "Unimplemented ai_condition_variable with variable type {:?}",
                variable_type
            );
            0
        }
        AipVariableType::Economy => {
            warn!(
                "Unimplemented ai_condition_variable with variable type {:?}",
                variable_type
            );
            0
        }
    };

    compare_aip_value(operator_type, variable_value, value)
}

fn ai_condition_server_channel_number(
    _ai_system_parameters: &AiSystemParameters,
    channel_range: &RangeInclusive<u16>,
) -> bool {
    // TODO: Do we need to have channel numbers?
    channel_range.contains(&1)
}

fn ai_condition_has_status_effect(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &AiParameters,
    who: AipHaveStatusTarget,
    status_effect_category: AipHaveStatusType,
    have: bool,
) -> bool {
    let status_effects = match who {
        AipHaveStatusTarget::This => Some(ai_parameters.source.status_effects),
        _ => ai_parameters
            .source
            .target
            .and_then(|target_entity| ai_system_parameters.target_query.get(target_entity).ok())
            .map(|(_, _, _, status_effects, _)| status_effects),
    };

    if let Some(status_effects) = status_effects {
        let mut has_any = false;
        let mut has_bad = false;
        let mut has_good = false;

        for (status_effect_type, active_status_effect) in status_effects.active.iter() {
            if active_status_effect.is_some() {
                has_any = true;

                if status_effect_type.is_good() {
                    has_good = true;
                }

                if status_effect_type.is_bad() {
                    has_bad = true;
                }
            }
        }

        match status_effect_category {
            AipHaveStatusType::Any => have == has_any,
            AipHaveStatusType::Good => have == has_good,
            AipHaveStatusType::Bad => have == has_bad,
        }
    } else {
        false
    }
}

fn get_aip_ability_value(
    ability_values: &AbilityValues,
    health_points: &HealthPoints,
    aip_ability_type: AipAbilityType,
) -> i32 {
    match aip_ability_type {
        AipAbilityType::Level => ability_values.get_level(),
        AipAbilityType::Attack => ability_values.get_attack_power(),
        AipAbilityType::Defence => ability_values.get_defence(),
        AipAbilityType::Resistance => ability_values.get_resistance(),
        AipAbilityType::HealthPoints => health_points.hp as i32,
        AipAbilityType::Charm => ability_values.get_charm(),
    }
}

fn ai_condition_target_ability_value(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &AiParameters,
    operator: AipOperatorType,
    aip_ability_type: AipAbilityType,
    value: i32,
) -> bool {
    if let Some((_, _, ability_values, _, health_points)) = ai_parameters
        .source
        .target
        .and_then(|target_entity| ai_system_parameters.target_query.get(target_entity).ok())
    {
        let ability_value = get_aip_ability_value(ability_values, health_points, aip_ability_type);
        compare_aip_value(operator, ability_value, value)
    } else {
        false
    }
}

fn ai_condition_attacker_and_target_ability_value(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &AiParameters,
    operator: AipOperatorType,
    aip_ability_type: AipAbilityType,
) -> bool {
    let attacker_ability_value = ai_parameters.attacker.map(|attacker_data| {
        get_aip_ability_value(
            attacker_data.ability_values,
            attacker_data.health_points,
            aip_ability_type,
        )
    });

    let target_ability_value = ai_parameters
        .source
        .target
        .and_then(|target_entity| ai_system_parameters.target_query.get(target_entity).ok())
        .map(|(_, _, ability_values, _, health_points)| {
            get_aip_ability_value(ability_values, health_points, aip_ability_type)
        });

    if let (Some(attacker_ability_value), Some(target_ability_value)) =
        (attacker_ability_value, target_ability_value)
    {
        compare_aip_value(operator, attacker_ability_value, target_ability_value)
    } else {
        false
    }
}

fn ai_condition_owner_has_target(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &AiParameters,
) -> bool {
    ai_parameters
        .source
        .owner
        .and_then(|owner_entity| ai_system_parameters.owner_query.get(owner_entity).ok())
        .map_or(false, |(_, target)| target.is_some())
}

fn npc_ai_check_conditions(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_program_event: &AipEvent,
    ai_parameters: &mut AiParameters,
) -> bool {
    for condition in ai_program_event.conditions.iter() {
        let result = match *condition {
            AipCondition::FindNearbyEntities(AipConditionFindNearbyEntities {
                distance,
                is_allied,
                ref level_diff_range,
                count_operator_type,
                count,
            }) => ai_condition_count_nearby_entities(
                ai_system_parameters,
                ai_parameters,
                distance,
                is_allied,
                level_diff_range,
                count_operator_type,
                count,
            )
            .is_ok(),
            AipCondition::Damage(damage_type, operator, value) => {
                ai_condition_damage(ai_parameters, damage_type, operator, value)
            }
            AipCondition::Distance(origin, operator, value) => {
                ai_condition_distance(ai_system_parameters, ai_parameters, origin, operator, value)
            }
            AipCondition::HasNoOwner => {
                ai_condition_has_no_owner(ai_system_parameters, ai_parameters)
            }
            AipCondition::HealthPercent(operator, value) => {
                ai_condition_health_percent(ai_parameters, operator, value)
            }
            AipCondition::IsAttackerCurrentTarget => {
                ai_condition_is_attacker_current_target(ai_parameters)
            }
            AipCondition::NoTargetAndCompareAttackerAbilityValue(operator, ability, value) => {
                ai_condition_no_target_and_compare_attacker_ability_value(
                    ai_parameters,
                    operator,
                    ability,
                    value,
                )
            }
            AipCondition::Random(operator, ref range, value) => {
                ai_condition_random(operator, range.clone(), value)
            }
            AipCondition::SelfAbilityValue(operator, ability, value) => {
                ai_condition_source_ability_value(ai_parameters, operator, ability, value)
            }
            AipCondition::SelectLocalNpc(npc_id) => {
                ai_condition_select_local_npc(ai_system_resources, ai_parameters, npc_id)
            }
            AipCondition::MonthDay(AipConditionMonthDayTime {
                month_day,
                ref day_minutes_range,
            }) => ai_condition_month_day_time(ai_system_resources, month_day, day_minutes_range),
            AipCondition::WeekDay(AipConditionWeekDayTime {
                week_day,
                ref day_minutes_range,
            }) => ai_condition_week_day_time(ai_system_resources, week_day, day_minutes_range),
            AipCondition::WorldTime(ref range) => {
                ai_condition_world_time(ai_system_resources, range)
            }
            AipCondition::ZoneTime(ref range) => {
                ai_condition_zone_time(ai_system_resources, ai_parameters, range)
            }
            AipCondition::IsDaytime(is_daytime) => {
                ai_condition_is_zone_daytime(ai_system_resources, ai_parameters, is_daytime)
            }
            AipCondition::Variable(variable_type, variable_id, operator_type, value) => {
                ai_condition_variable(
                    ai_system_parameters,
                    ai_parameters,
                    variable_type,
                    variable_id,
                    operator_type,
                    value,
                )
            }
            AipCondition::ServerChannelNumber(ref channel_range) => {
                ai_condition_server_channel_number(ai_system_parameters, channel_range)
            }
            AipCondition::HasStatusEffect(who, status_effect_category, have) => {
                ai_condition_has_status_effect(
                    ai_system_parameters,
                    ai_parameters,
                    who,
                    status_effect_category,
                    have,
                )
            }
            AipCondition::TargetAbilityValue(operator, aip_ability_type, value) => {
                ai_condition_target_ability_value(
                    ai_system_parameters,
                    ai_parameters,
                    operator,
                    aip_ability_type,
                    value,
                )
            }
            AipCondition::CompareAttackerAndTargetAbilityValue(operator, aip_ability_type) => {
                ai_condition_attacker_and_target_ability_value(
                    ai_system_parameters,
                    ai_parameters,
                    operator,
                    aip_ability_type,
                )
            }
            AipCondition::OwnerHasTarget => {
                ai_condition_owner_has_target(ai_system_parameters, ai_parameters)
            }
            /*
            AipCondition::IsAttackerClanMaster => false,
            AipCondition::IsTargetClanMaster => false,
            */
            _ => {
                warn!("Unimplemented AI condition: {:?}", condition);
                false
            }
        };

        if !result {
            return false;
        }
    }

    true
}

fn ai_action_stop(ai_system_parameters: &mut AiSystemParameters, ai_parameters: &mut AiParameters) {
    ai_system_parameters
        .commands
        .entity(ai_parameters.source.entity)
        .insert(NextCommand::with_stop(true));
}

fn ai_action_move_random_distance(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    move_origin: AipMoveOrigin,
    move_mode: AipMoveMode,
    distance: i32,
) {
    let dx = rand::thread_rng().gen_range(-distance..distance);
    let dy = rand::thread_rng().gen_range(-distance..distance);
    let move_origin = match move_origin {
        AipMoveOrigin::CurrentPosition => Some(ai_parameters.source.position.position),
        AipMoveOrigin::Spawn => {
            ai_parameters
                .source
                .spawn_origin
                .map(|spawn_origin| match *spawn_origin {
                    SpawnOrigin::MonsterSpawnPoint(_, spawn_position) => spawn_position,
                    SpawnOrigin::Summoned(_, spawn_position) => spawn_position,
                    SpawnOrigin::Quest(_, spawn_position) => spawn_position,
                })
        }
        AipMoveOrigin::FindChar => ai_parameters.find_char.map(|(_, position)| position),
    };

    if let Some(move_origin) = move_origin {
        let move_mode = match move_mode {
            AipMoveMode::Run => MoveMode::Run,
            AipMoveMode::Walk => MoveMode::Walk,
        };
        let destination = move_origin + Vector3::new(dx as f32, dy as f32, 0.0);
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_move(destination, None, Some(move_mode)));
    }
}

fn ai_action_move_near_owner(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    if let Some(owner_position) = ai_parameters
        .source
        .owner
        .and_then(|owner_entity| ai_system_parameters.owner_query.get(owner_entity).ok())
        .map(|(position, _)| position.clone())
    {
        let distance = (0.2
            * nalgebra::distance(
                &owner_position.position.xy(),
                &ai_parameters.source.position.position.xy(),
            )) as i32;
        let dx = rand::thread_rng().gen_range(-distance..distance);
        let dy = rand::thread_rng().gen_range(-distance..distance);

        let destination = owner_position.position + Vector3::new(dx as f32, dy as f32, 0.0);
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_move(
                destination,
                None,
                Some(MoveMode::Run),
            ));
    }
}

fn npc_ai_do_actions(
    ai_system_parameters: &mut AiSystemParameters,
    ai_program_event: &AipEvent,
    ai_parameters: &mut AiParameters,
) {
    for action in ai_program_event.actions.iter() {
        match *action {
            AipAction::Stop => ai_action_stop(ai_system_parameters, ai_parameters),
            AipAction::MoveRandomDistance(move_origin, move_mode, distance) => {
                ai_action_move_random_distance(
                    ai_system_parameters,
                    ai_parameters,
                    move_origin,
                    move_mode,
                    distance,
                )
            }
            AipAction::AttackNearChar => {
                if let Some((near_char, _)) = ai_parameters.near_char {
                    ai_system_parameters
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(near_char));
                }
            }
            AipAction::AttackFindChar => {
                if let Some((find_char, _)) = ai_parameters.find_char {
                    ai_system_parameters
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(find_char));
                }
            }
            AipAction::AttackAttacker => {
                if let Some(attacker) = ai_parameters.attacker {
                    ai_system_parameters
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(attacker.entity));
                }
            }
            AipAction::KillSelf => {
                // TODO: Fix this, this doesn't send death to clients.
                ai_system_parameters
                    .commands
                    .entity(ai_parameters.source.entity)
                    .insert(HealthPoints::new(0))
                    .insert(Command::with_die(None, None, None));
            }
            AipAction::MoveNearOwner => {
                ai_action_move_near_owner(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackOwnerTarget => {
                if let Some(owner_target_entity) = ai_parameters
                    .source
                    .owner
                    .and_then(|owner_entity| {
                        ai_system_parameters.owner_query.get(owner_entity).ok()
                    })
                    .and_then(|(_, target)| target.map(|target| target.entity))
                {
                    ai_system_parameters
                        .commands
                        .entity(ai_parameters.source.entity)
                        .insert(NextCommand::with_attack(owner_target_entity));
                }
            }
            /*
            AipAction::Emote(_) => {}
            AipAction::Say(_) => {}
            AipAction::AttackNearbyEntityByStat(_, _, _) => {}
            AipAction::SpecialAttack => {}
            AipAction::MoveDistanceFromTarget(_, _) => {}
            AipAction::TransformNpc(_) => {}
            AipAction::SpawnNpc(_, _, _, _) => {}
            AipAction::NearbyAlliesAttackTarget(_, _, _) => {}
            AipAction::NearbyAlliesSameNpcAttackTarget(_) => {}
            AipAction::RunAway(_) => {}
            AipAction::DropRandomItem(_) => {
            AipAction::UseSkill(_, _, _) => {}
            AipAction::SetVariable(_, _, _, _) => {}
            AipAction::Message(_, _) => {}
            AipAction::DoQuestTrigger(_) => {}
            AipAction::SetPvpFlag(_, _) => {}
            AipAction::SetMonsterSpawnState(_, _) => {}
            AipAction::GiveItemToOwner(_, _) => {}
            */
            _ => {
                trace!("Unimplemented AI action: {:?}", action);
            }
        }
    }
}

fn npc_ai_run_trigger(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_trigger: &AipTrigger,
    source: &AiSourceData,
    attacker: Option<AiAttackerData>,
    damage: Option<Damage>,
    is_dead: bool,
) {
    let mut ai_parameters = AiParameters {
        source,
        attacker: attacker.as_ref(),
        find_char: None,
        near_char: None,
        selected_local_npc: None,
        damage_received: damage,
        is_dead,
    };

    // Do actions for only the first event with valid conditions
    for ai_program_event in ai_trigger.events.iter() {
        if npc_ai_check_conditions(
            ai_system_parameters,
            ai_system_resources,
            ai_program_event,
            &mut ai_parameters,
        ) {
            npc_ai_do_actions(ai_system_parameters, ai_program_event, &mut ai_parameters);
            break;
        }
    }
}

fn get_attacker_data<'w, 's>(
    attacker_query: &Query<'w, 's, (&Position, &Level, &Team, &AbilityValues, &HealthPoints)>,
    entity: Entity,
) -> Option<AiAttackerData<'w>> {
    if let Ok((
        attacker_position,
        attacker_level,
        attacker_team,
        attacker_ability_values,
        attacker_health_points,
    )) = attacker_query.get(entity)
    {
        Some(AiAttackerData::<'w> {
            entity,
            _position: attacker_position,
            _team: attacker_team,
            ability_values: attacker_ability_values,
            health_points: attacker_health_points,
            level: attacker_level,
        })
    } else {
        None
    }
}

pub fn npc_ai_system(
    mut ai_system_parameters: AiSystemParameters,
    ai_system_resources: AiSystemResources,
    mut npc_query: Query<(
        Entity,
        &Npc,
        &mut NpcAi,
        &ClientEntity,
        &ClientEntitySector,
        &Command,
        &Position,
        &Level,
        &Team,
        &HealthPoints,
        &AbilityValues,
        &StatusEffects,
        (
            Option<&Owner>,
            Option<&SpawnOrigin>,
            Option<&Target>,
            Option<&DamageSources>,
        ),
    )>,
    mut spawn_point_query: Query<&mut MonsterSpawnPoint>,
    attacker_query: Query<(&Position, &Level, &Team, &AbilityValues, &HealthPoints)>,
    killer_query: Query<(&Level, &AbilityValues, Option<&GameClient>)>,
    world_rates: Res<WorldRates>,
    mut reward_xp_events: EventWriter<RewardXpEvent>,
) {
    npc_query.for_each_mut(
        |(
            entity,
            npc,
            mut npc_ai,
            client_entity,
            client_entity_sector,
            command,
            position,
            level,
            team,
            health_points,
            ability_values,
            status_effects,
            (owner, spawn_origin, target, damage_sources),
        )| {
            let ai_source_data = AiSourceData {
                entity,
                ability_values,
                health_points,
                level,
                owner: owner.map(|owner| owner.entity),
                position,
                spawn_origin,
                status_effects,
                target: target.map(|target| target.entity),
                team,
            };

            if !npc_ai.has_run_created_trigger {
                if let Some(ai_program) = ai_system_resources.game_data.ai.get_ai(npc_ai.ai_index) {
                    if let Some(trigger_on_created) = ai_program.trigger_on_created.as_ref() {
                        npc_ai_run_trigger(
                            &mut ai_system_parameters,
                            &ai_system_resources,
                            trigger_on_created,
                            &ai_source_data,
                            None,
                            None,
                            false,
                        );
                    }
                }

                (*npc_ai).has_run_created_trigger = true;
            }

            if let Some(ai_program) = ai_system_resources.game_data.ai.get_ai(npc_ai.ai_index) {
                if let Some(trigger_on_damaged) = ai_program.trigger_on_damaged.as_ref() {
                    let mut rng = rand::thread_rng();
                    for &(attacker_entity, damage) in npc_ai.pending_damage.iter() {
                        if command.get_target().is_some()
                            && ai_program.damage_trigger_new_target_chance < rng.gen_range(0..100)
                        {
                            continue;
                        }

                        if let Some(attacker_data) =
                            get_attacker_data(&attacker_query, attacker_entity)
                        {
                            npc_ai_run_trigger(
                                &mut ai_system_parameters,
                                &ai_system_resources,
                                trigger_on_damaged,
                                &ai_source_data,
                                Some(attacker_data),
                                Some(damage),
                                false,
                            );
                        }
                    }
                }
            }
            npc_ai.pending_damage.clear();

            match command.command {
                CommandData::Stop(_) => {
                    if let Some(ai_program) =
                        ai_system_resources.game_data.ai.get_ai(npc_ai.ai_index)
                    {
                        if let Some(trigger_on_idle) = ai_program.trigger_on_idle.as_ref() {
                            npc_ai.idle_duration += ai_system_resources.server_time.delta;

                            if npc_ai.idle_duration > ai_program.idle_trigger_interval {
                                npc_ai_run_trigger(
                                    &mut ai_system_parameters,
                                    &ai_system_resources,
                                    trigger_on_idle,
                                    &ai_source_data,
                                    None,
                                    None,
                                    false,
                                );
                                npc_ai.idle_duration -= ai_program.idle_trigger_interval;
                            }
                        }
                    }
                }
                CommandData::Die(CommandDie {
                    killer: killer_entity,
                    damage: killer_damage,
                }) => {
                    if !npc_ai.has_run_dead_ai {
                        npc_ai.has_run_dead_ai = true;

                        // Notify spawn point that one of it's monsters died
                        if let Some(&SpawnOrigin::MonsterSpawnPoint(spawn_point_entity, _)) =
                            spawn_origin
                        {
                            if let Ok(mut spawn_point) =
                                spawn_point_query.get_mut(spawn_point_entity)
                            {
                                let mut spawn_point = &mut *spawn_point;
                                spawn_point.num_alive_monsters =
                                    spawn_point.num_alive_monsters.saturating_sub(1);
                            }
                        }

                        // Run on dead AI
                        if let Some(trigger_on_dead) = ai_system_resources
                            .game_data
                            .ai
                            .get_ai(npc_ai.ai_index)
                            .and_then(|ai_program| ai_program.trigger_on_dead.as_ref())
                        {
                            let attacker_data = killer_entity.and_then(|killer_entity| {
                                get_attacker_data(&attacker_query, killer_entity)
                            });

                            npc_ai_run_trigger(
                                &mut ai_system_parameters,
                                &ai_system_resources,
                                trigger_on_dead,
                                &ai_source_data,
                                attacker_data,
                                killer_damage,
                                true,
                            );
                        }

                        if let Some(damage_sources) = damage_sources {
                            if let Some(npc_data) =
                                ai_system_resources.game_data.npcs.get_npc(npc.id)
                            {
                                // Reward XP to all attackers
                                for damage_source in damage_sources.damage_sources.iter() {
                                    let time_since_damage = ai_system_resources.server_time.now
                                        - damage_source.last_damage_time;
                                    if time_since_damage > DAMAGE_REWARD_EXPIRE_TIME {
                                        // Damage expired, ignore.
                                        continue;
                                    }

                                    if let Ok((damage_source_level, ..)) =
                                        ai_system_parameters.target_query.get(damage_source.entity)
                                    {
                                        let reward_xp = ai_system_resources
                                            .game_data
                                            .ability_value_calculator
                                            .calculate_give_xp(
                                                damage_source_level.level as i32,
                                                damage_source.total_damage as i32,
                                                level.level as i32,
                                                ability_values.get_max_health(),
                                                npc_data.reward_xp as i32,
                                                world_rates.xp_rate,
                                            );
                                        if reward_xp > 0 {
                                            let stamina = ai_system_resources
                                                .game_data
                                                .ability_value_calculator
                                                .calculate_give_stamina(
                                                    reward_xp,
                                                    level.level as i32,
                                                    world_rates.xp_rate,
                                                );

                                            reward_xp_events.send(RewardXpEvent::new(
                                                damage_source.entity,
                                                reward_xp as u64,
                                                stamina as u32,
                                                Some(entity),
                                            ));
                                        }
                                    }
                                }

                                // Reward killer
                                if let Some(killer_entity) = killer_entity {
                                    if let Ok((
                                        killer_level,
                                        killer_ability_values,
                                        killer_game_client,
                                    )) = killer_query.get(killer_entity)
                                    {
                                        // Inform client to execute npc dead event
                                        if !npc_data.death_quest_trigger_name.is_empty() {
                                            if let Some(killer_game_client) = killer_game_client {
                                                // TODO: Send NPC death trigger to whole party
                                                /*
                                                if npc_data.npc_quest_type != 0 {
                                                }
                                                */

                                                // Send to only client
                                                killer_game_client
                                                    .server_message_tx
                                                    .send(ServerMessage::RunNpcDeathTrigger(npc.id))
                                                    .ok();
                                            }
                                        }

                                        // Drop item owned by killer
                                        let level_difference =
                                            killer_level.level as i32 - level.level as i32;
                                        if let Some(drop_item) =
                                            ai_system_resources.game_data.drop_table.get_drop(
                                                world_rates.drop_rate,
                                                world_rates.drop_money_rate,
                                                npc.id,
                                                position.zone_id,
                                                level_difference,
                                                killer_ability_values.get_drop_rate(),
                                                killer_ability_values.get_charm(),
                                            )
                                        {
                                            ItemDropBundle::spawn(
                                                &mut ai_system_parameters.commands,
                                                &mut ai_system_parameters.client_entity_list,
                                                drop_item,
                                                position,
                                                Some(killer_entity),
                                                &ai_system_resources.server_time,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Once the death animation has completed, we can remove this entity
                    let command_complete = command
                        .required_duration
                        .map_or(false, |required_duration| {
                            command.duration >= required_duration
                        });
                    if command_complete {
                        client_entity_leave_zone(
                            &mut ai_system_parameters.commands,
                            &mut ai_system_parameters.client_entity_list,
                            entity,
                            client_entity,
                            client_entity_sector,
                            position,
                        );
                        ai_system_parameters.commands.entity(entity).despawn();
                    }
                }
                _ => {}
            }
        },
    );
}
