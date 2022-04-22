use bevy::ecs::{
    prelude::{Commands, Entity, EventWriter, Query, Res, ResMut},
    system::SystemParam,
};
use bevy::math::{Vec3, Vec3Swizzles};
use chrono::{Datelike, Timelike};
use rand::{prelude::SliceRandom, Rng};
use std::{
    marker::PhantomData,
    num::NonZeroU8,
    ops::{Range, RangeInclusive},
    time::Duration,
};

use rose_data::{Item, MotionId, NpcId, SkillId, ZoneId};
use rose_file_readers::{
    AipAbilityType, AipAction, AipAttackNearbyStat, AipCondition, AipConditionFindNearbyEntities,
    AipConditionMonthDayTime, AipConditionWeekDayTime, AipDamageType, AipDistance,
    AipDistanceOrigin, AipEvent, AipHaveStatusTarget, AipHaveStatusType, AipItemBase1000,
    AipMessageType, AipMonsterSpawnState, AipMotionId, AipMoveMode, AipMoveOrigin, AipNearbyAlly,
    AipNpcId, AipOperatorType, AipResultOperator, AipSkillId, AipSkillTarget, AipSpawnNpcOrigin,
    AipTrigger, AipVariableType, AipZoneId,
};
use rose_game_common::data::Damage;

use crate::game::{
    bundles::{client_entity_leave_zone, ItemDropBundle, MonsterBundle},
    components::{
        AbilityValues, ClientEntity, ClientEntitySector, ClientEntityType, Command, CommandData,
        CommandDie, DamageSources, DroppedItem, GameClient, HealthPoints, Level, MonsterSpawnPoint,
        MoveMode, NextCommand, Npc, NpcAi, ObjectVariables, Owner, Position, SpawnOrigin,
        StatusEffects, Target, Team,
    },
    events::{DamageEvent, QuestTriggerEvent, RewardItemEvent, RewardXpEvent},
    messages::server::{AnnounceChat, LocalChat, ServerMessage, ShoutChat},
    resources::{ClientEntityList, ServerMessages, ServerTime, WorldRates, WorldTime, ZoneList},
    GameData,
};

const DAMAGE_REWARD_EXPIRE_TIME: Duration = Duration::from_secs(5 * 60);

#[derive(SystemParam)]
pub struct AiSystemParameters<'w, 's> {
    commands: Commands<'w, 's>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    server_messages: ResMut<'w, ServerMessages>,
    target_query: Query<
        'w,
        's,
        (
            &'static Level,
            &'static Team,
            &'static AbilityValues,
            &'static StatusEffects,
            &'static HealthPoints,
            &'static Position,
            Option<&'static Target>,
            Option<&'static Npc>,
        ),
    >,
    object_variable_query: Query<'w, 's, &'static mut ObjectVariables>,
    owner_query: Query<'w, 's, (&'static Position, Option<&'static Target>)>,
    damage_events: EventWriter<'w, 's, DamageEvent>,
    quest_trigger_events: EventWriter<'w, 's, QuestTriggerEvent>,
    reward_item_events: EventWriter<'w, 's, RewardItemEvent>,
    zone_list: ResMut<'w, ZoneList>,
}

#[derive(SystemParam)]
pub struct AiSystemResources<'w, 's> {
    game_data: Res<'w, GameData>,
    server_time: Res<'w, ServerTime>,
    world_time: Res<'w, WorldTime>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

struct AiSourceData<'s> {
    entity: Entity,
    npc: &'s Npc,
    client_entity: &'s ClientEntity,
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
    position: &'a Position,
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
    find_char: Option<(Entity, Vec3)>,
    near_char: Option<(Entity, Vec3)>,
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
        let distance_squared = ai_parameters
            .source
            .position
            .position
            .distance_squared(position);
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
        ai_parameters
            .source
            .position
            .position
            .xy()
            .distance_squared(compare_position) as i32
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
    ai_system_parameters: &AiSystemParameters,
    ai_parameters: &mut AiParameters,
    npc_id: AipNpcId,
) -> bool {
    let local_npc = NpcId::new(npc_id as u16)
        .and_then(|npc_id| ai_system_parameters.zone_list.find_npc(npc_id));
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
            log::warn!(target: "npc_ai_unimplemented",
                "Unimplemented ai_condition_variable with variable type {:?}",
                variable_type
            );
            0
        }
        AipVariableType::Economy => {
            log::warn!(target: "npc_ai_unimplemented",
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
            .map(|(_, _, _, status_effects, _, _, _, _)| status_effects),
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
    if let Some((_, _, ability_values, _, health_points, _, _, _)) = ai_parameters
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
        .map(|(_, _, ability_values, _, health_points, _, _, _)| {
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
                ai_condition_select_local_npc(ai_system_parameters, ai_parameters, npc_id)
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
                log::warn!(target: "npc_ai_unimplemented", "  - Unimplemented AI condition: {:?}", condition);
                false
            }
        };
        log::trace!(target: "npc_ai", "  - AI condition: {:?} = {}", condition, result);

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

fn ai_action_attack_attacker(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    if let Some(attacker) = ai_parameters.attacker {
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_attack(attacker.entity));
    }
}

fn ai_action_attack_find_char(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    if let Some((find_char, _)) = ai_parameters.find_char {
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_attack(find_char));
    }
}

fn ai_action_attack_near_char(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    if let Some((near_char, _)) = ai_parameters.near_char {
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_attack(near_char));
    }
}

fn ai_action_move_away_from_target(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    move_mode: AipMoveMode,
    distance: i32,
) {
    let move_mode = match move_mode {
        AipMoveMode::Run => MoveMode::Run,
        AipMoveMode::Walk => MoveMode::Walk,
    };

    if let Some(target_entity) = ai_parameters.source.target {
        if let Ok((_, _, _, _, _, target_position, _, _)) =
            ai_system_parameters.target_query.get(target_entity)
        {
            let source_position = ai_parameters.source.position.position;
            let direction_away_from_target =
                (source_position.xy() - target_position.position.xy()).normalize();
            let move_vector = distance as f32 * direction_away_from_target;
            let destination = source_position + Vec3::new(move_vector.x, move_vector.y, 0.0);

            ai_system_parameters
                .commands
                .entity(ai_parameters.source.entity)
                .insert(NextCommand::with_move(destination, None, Some(move_mode)));
        }
    }
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
        let destination = move_origin + Vec3::new(dx as f32, dy as f32, 0.0);
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
        // Move 80% of the way towards owner
        let delta = owner_position.position.xy() - ai_parameters.source.position.position.xy();
        let distance = 0.8 * delta.length();
        let direction = delta.normalize();
        let destination = ai_parameters.source.position.position.xy() + direction * distance;

        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_move(
                Vec3::new(destination.x, destination.y, 0.0),
                None,
                Some(MoveMode::Run),
            ));
    }
}

fn ai_action_attack_owner_target(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    if let Some(owner_target_entity) = ai_parameters
        .source
        .owner
        .and_then(|owner_entity| ai_system_parameters.owner_query.get(owner_entity).ok())
        .and_then(|(_, target)| target.map(|target| target.entity))
    {
        if let Ok((_, target_team, _, _, _, _, _, _)) =
            ai_system_parameters.target_query.get(owner_target_entity)
        {
            if target_team.id != Team::DEFAULT_NPC_TEAM_ID
                && target_team.id != ai_parameters.source.team.id
            {
                ai_system_parameters
                    .commands
                    .entity(ai_parameters.source.entity)
                    .insert(NextCommand::with_attack(owner_target_entity));
            }
        }
    }
}

fn ai_action_attack_nearby_entity_by_stat(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    distance: AipDistance,
    ability_type: AipAbilityType,
    stat_choice: AipAttackNearbyStat,
) {
    let zone_entities = ai_system_parameters
        .client_entity_list
        .get_zone(ai_parameters.source.position.zone_id);
    if zone_entities.is_none() {
        return;
    }

    let mut min_entity = None;
    let mut max_entity = None;

    for (entity, _) in zone_entities
        .unwrap()
        .iter_entities_within_distance(ai_parameters.source.position.position.xy(), distance as f32)
    {
        if entity == ai_parameters.source.entity {
            continue;
        }

        if let Ok((level, team, ability_values, _, health_points, _, _, _)) =
            ai_system_parameters.target_query.get(entity)
        {
            if team.id != Team::DEFAULT_NPC_TEAM_ID && team.id != ai_parameters.source.team.id {
                let value = match ability_type {
                    AipAbilityType::Level => level.level as i32,
                    AipAbilityType::Attack => ability_values.get_attack_power(),
                    AipAbilityType::Defence => ability_values.get_defence(),
                    AipAbilityType::Resistance => ability_values.get_resistance(),
                    AipAbilityType::HealthPoints => health_points.hp,
                    AipAbilityType::Charm => ability_values.get_charm(),
                };

                if min_entity.map_or(true, |(_, min_value)| value < min_value) {
                    min_entity = Some((entity, value));
                }

                if max_entity.map_or(true, |(_, max_value)| value > max_value) {
                    max_entity = Some((entity, value));
                }
            }
        }
    }

    let target_entity = match stat_choice {
        AipAttackNearbyStat::Lowest => min_entity.map(|(entity, _)| entity),
        AipAttackNearbyStat::Highest => max_entity.map(|(entity, _)| entity),
    };

    if let Some(target_entity) = target_entity {
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(NextCommand::with_attack(target_entity));
    }
}

fn ai_action_quest_trigger(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    trigger_name: &str,
) {
    let trigger_hash = trigger_name.into();

    if matches!(
        ai_parameters.source.client_entity.entity_type,
        ClientEntityType::Monster
    ) {
        if let Some(entity) = ai_parameters.selected_local_npc {
            ai_system_parameters
                .quest_trigger_events
                .send(QuestTriggerEvent {
                    trigger_entity: entity,
                    trigger_hash,
                });
        }
    } else {
        ai_system_parameters
            .quest_trigger_events
            .send(QuestTriggerEvent {
                trigger_entity: ai_parameters.source.entity,
                trigger_hash,
            });
    }
}

fn ai_action_kill_self(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
) {
    ai_system_parameters
        .damage_events
        .send(DamageEvent::with_attack(
            ai_parameters.source.entity,
            ai_parameters.source.entity,
            Damage {
                amount: ai_parameters.source.health_points.hp as u32 + 1,
                is_critical: false,
                apply_hit_stun: false,
            },
        ));
}

fn ai_action_nearby_allies_attack_target(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    distance: AipDistance,
    nearby_ally_type: AipNearbyAlly,
    limit: Option<usize>,
) {
    let target_entity = ai_parameters.source.target;
    if target_entity.is_none() {
        return;
    }
    let target_entity = target_entity.unwrap();

    let zone_entities = ai_system_parameters
        .client_entity_list
        .get_zone(ai_parameters.source.position.zone_id);
    if zone_entities.is_none() {
        return;
    }

    let mut num_attackers = 0;

    for (nearby_entity, _) in zone_entities
        .unwrap()
        .iter_entities_within_distance(ai_parameters.source.position.position.xy(), distance as f32)
    {
        if nearby_entity == ai_parameters.source.entity {
            continue;
        }

        if let Ok((_, nearby_team, _, _, _, _, nearby_target, nearby_npc)) =
            ai_system_parameters.target_query.get(nearby_entity)
        {
            if nearby_target.is_some()
                || nearby_team.id != ai_parameters.source.team.id
                || nearby_npc.is_none()
            {
                continue;
            }

            let nearby_npc = nearby_npc.unwrap();
            let valid = match nearby_ally_type {
                AipNearbyAlly::Ally => true,
                AipNearbyAlly::WithNpcId(npc_id) => nearby_npc.id.get() == npc_id as u16,
                AipNearbyAlly::WithSameNpcId => nearby_npc.id == ai_parameters.source.npc.id,
            };
            if !valid {
                continue;
            }

            ai_system_parameters
                .commands
                .entity(nearby_entity)
                .insert(NextCommand::with_attack(target_entity));

            num_attackers += 1;
        }

        if let Some(limit) = limit {
            if num_attackers == limit {
                break;
            }
        }
    }
}

fn ai_action_spawn_npc(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_parameters: &mut AiParameters,
    npc_id: AipNpcId,
    distance: AipDistance,
    origin: AipSpawnNpcOrigin,
    is_owner: bool,
) {
    let spawn_position = match origin {
        AipSpawnNpcOrigin::CurrentPosition => Some(ai_parameters.source.position.position),
        AipSpawnNpcOrigin::AttackerPosition => ai_parameters
            .attacker
            .map(|attacker| attacker.position.position),
        AipSpawnNpcOrigin::TargetPosition => {
            ai_parameters.source.target.and_then(|target_entity| {
                ai_system_parameters
                    .target_query
                    .get(target_entity)
                    .map(|(_, _, _, _, _, position, _, _)| position.position)
                    .ok()
            })
        }
    };

    let npc_id = NpcId::new(npc_id as u16);
    if npc_id.is_none() {
        return;
    }

    if let Some(spawn_position) = spawn_position {
        // TODO: If ai_parameters.is_dead { spawn after 3 seconds }
        if let Some(spawn_entity) = MonsterBundle::spawn(
            &mut ai_system_parameters.commands,
            &mut ai_system_parameters.client_entity_list,
            &ai_system_resources.game_data,
            npc_id.unwrap(),
            ai_parameters.source.position.zone_id,
            SpawnOrigin::Summoned(ai_parameters.source.entity, spawn_position),
            distance,
            ai_parameters.source.team.clone(),
            None,
            None,
        ) {
            if is_owner {
                ai_system_parameters
                    .commands
                    .entity(spawn_entity)
                    .insert(Owner::new(ai_parameters.source.entity));
            }
        }
    }
}

fn ai_action_transform_npc(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    npc_id: AipNpcId,
) {
    if let Some(npc_id) = NpcId::new(npc_id as u16) {
        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(Npc::new(npc_id, 0));

        ai_system_parameters.server_messages.send_entity_message(
            ai_parameters.source.client_entity,
            ServerMessage::ChangeNpcId(ai_parameters.source.client_entity.id, npc_id),
        );
    }
}

fn ai_action_use_emote(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    motion_id: AipMotionId,
) {
    let motion_id = MotionId::new(motion_id as u16);

    ai_system_parameters
        .commands
        .entity(ai_parameters.source.entity)
        .insert(NextCommand::with_emote(motion_id, true));
}

fn ai_action_use_skill(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    target: AipSkillTarget,
    skill_id: AipSkillId,
    motion_id: AipMotionId,
) {
    let target_entity = match target {
        AipSkillTarget::FindChar => ai_parameters.find_char.map(|(entity, _)| entity),
        AipSkillTarget::Target => ai_parameters.source.target,
        AipSkillTarget::This => Some(ai_parameters.source.entity),
        AipSkillTarget::NearChar => ai_parameters.near_char.map(|(entity, _)| entity),
    };
    let skill_id = SkillId::new(skill_id as u16);
    let cast_motion_id = MotionId::new(motion_id as u16);
    let action_motion_id = MotionId::new(motion_id as u16 + 1);

    if let (Some(skill_id), Some(target_entity)) = (skill_id, target_entity) {
        let next_command = if target_entity != ai_parameters.source.entity {
            NextCommand::with_npc_cast_skill_target(
                skill_id,
                target_entity,
                cast_motion_id,
                action_motion_id,
            )
        } else {
            NextCommand::with_npc_cast_skill_self(skill_id, cast_motion_id, action_motion_id)
        };

        ai_system_parameters
            .commands
            .entity(ai_parameters.source.entity)
            .insert(next_command);
    }
}

fn ai_action_set_monster_spawn_state(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    zone_id: Option<AipZoneId>,
    state: AipMonsterSpawnState,
) {
    let zone_id = zone_id
        .and_then(|zone_id| ZoneId::new(zone_id as u16))
        .unwrap_or(ai_parameters.source.position.zone_id);

    let enabled = match state {
        AipMonsterSpawnState::Disabled => false,
        AipMonsterSpawnState::Enabled => true,
        AipMonsterSpawnState::Toggle => !ai_system_parameters
            .zone_list
            .get_monster_spawns_enabled(zone_id),
    };

    ai_system_parameters
        .zone_list
        .set_monster_spawns_enabled(zone_id, enabled);
}

fn ai_action_set_variable(
    ai_system_parameters: &mut AiSystemParameters,
    ai_parameters: &mut AiParameters,
    variable_type: AipVariableType,
    variable_id: usize,
    operator: AipResultOperator,
    value: i32,
) {
    match variable_type {
        AipVariableType::LocalNpcObject => ai_parameters
            .selected_local_npc
            .and_then(|object_entity| {
                ai_system_parameters
                    .object_variable_query
                    .get_mut(object_entity)
                    .ok()
            })
            .map(|mut object_variables| {
                object_variables
                    .variables
                    .get_mut(variable_id)
                    .map(|variable| match operator {
                        AipResultOperator::Set => *variable = value,
                        AipResultOperator::Add => *variable = i32::min(*variable + value, 500),
                        AipResultOperator::Subtract => *variable = i32::max(*variable - value, 0),
                    })
            }),
        AipVariableType::Ai => ai_system_parameters
            .object_variable_query
            .get_mut(ai_parameters.source.entity)
            .ok()
            .map(|mut object_variables| {
                object_variables
                    .variables
                    .get_mut(variable_id)
                    .map(|variable| match operator {
                        AipResultOperator::Set => *variable = value,
                        AipResultOperator::Add => *variable += value,
                        AipResultOperator::Subtract => *variable -= value,
                    })
            }),
        AipVariableType::World => {
            log::warn!(target: "npc_ai_unimplemented",
                "Unimplemented ai_action_set_variable with variable type {:?}",
                variable_type
            );
            None
        }
        AipVariableType::Economy => {
            log::warn!(target: "npc_ai_unimplemented",
                "Unimplemented ai_action_set_variable with variable type {:?}",
                variable_type
            );
            None
        }
    };
}
fn ai_action_message(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_parameters: &mut AiParameters,
    message_type: AipMessageType,
    string_id: usize,
) {
    let npc_name = ai_system_resources
        .game_data
        .npcs
        .get_npc(ai_parameters.source.npc.id)
        .map(|npc_data| npc_data.name.clone());

    if let Some(message) = ai_system_resources.game_data.ai.get_ai_string(string_id) {
        match message_type {
            AipMessageType::Say => ai_system_parameters.server_messages.send_entity_message(
                ai_parameters.source.client_entity,
                ServerMessage::LocalChat(LocalChat {
                    entity_id: ai_parameters.source.client_entity.id,
                    text: message.to_string(),
                }),
            ),
            AipMessageType::Shout => {
                if let Some(npc_name) = npc_name {
                    ai_system_parameters.server_messages.send_entity_message(
                        ai_parameters.source.client_entity,
                        ServerMessage::ShoutChat(ShoutChat {
                            name: npc_name,
                            text: message.to_string(),
                        }),
                    )
                }
            }
            AipMessageType::Announce => {
                if let Some(npc_name) = npc_name {
                    ai_system_parameters.server_messages.send_entity_message(
                        ai_parameters.source.client_entity,
                        ServerMessage::AnnounceChat(AnnounceChat {
                            name: Some(npc_name),
                            text: message.to_string(),
                        }),
                    )
                }
            }
        }
    }
}

fn ai_action_drop_random_item(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_parameters: &mut AiParameters,
    items_base1000: &[AipItemBase1000],
) {
    if let Some(item) = items_base1000
        .choose(&mut rand::thread_rng())
        .and_then(|item_base1000| {
            ai_system_resources
                .game_data
                .data_decoder
                .decode_item_base1000(*item_base1000 as usize)
        })
        .and_then(|item_reference| {
            ai_system_resources
                .game_data
                .items
                .get_base_item(item_reference)
        })
        .and_then(|item_data| Item::from_item_data(item_data, 1))
    {
        ItemDropBundle::spawn(
            &mut ai_system_parameters.commands,
            &mut ai_system_parameters.client_entity_list,
            DroppedItem::Item(item),
            ai_parameters.source.position,
            None,
            &ai_system_resources.server_time,
        );
    }
}

fn ai_action_give_item_to_owner(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_parameters: &mut AiParameters,
    item_base1000: AipItemBase1000,
    quantity: usize,
) {
    if let Some(item) = ai_system_resources
        .game_data
        .data_decoder
        .decode_item_base1000(item_base1000 as usize)
        .and_then(|item_reference| {
            ai_system_resources
                .game_data
                .items
                .get_base_item(item_reference)
        })
        .and_then(|item_data| Item::from_item_data(item_data, quantity as u32))
    {
        ai_system_parameters
            .reward_item_events
            .send(RewardItemEvent::new(
                ai_parameters.source.entity,
                item,
                true,
            ));
    }
}

fn npc_ai_do_actions(
    ai_system_parameters: &mut AiSystemParameters,
    ai_system_resources: &AiSystemResources,
    ai_program_event: &AipEvent,
    ai_parameters: &mut AiParameters,
) {
    for action in ai_program_event.actions.iter() {
        log::trace!(target: "npc_ai", "  - AI action: {:?}", action);
        match *action {
            AipAction::Stop => ai_action_stop(ai_system_parameters, ai_parameters),
            AipAction::MoveAwayFromTarget(move_mode, distance) => ai_action_move_away_from_target(
                ai_system_parameters,
                ai_parameters,
                move_mode,
                distance,
            ),
            AipAction::MoveRandomDistance(move_origin, move_mode, distance) => {
                ai_action_move_random_distance(
                    ai_system_parameters,
                    ai_parameters,
                    move_origin,
                    move_mode,
                    distance,
                )
            }
            AipAction::MoveNearOwner => {
                ai_action_move_near_owner(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackNearChar => {
                ai_action_attack_near_char(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackFindChar => {
                ai_action_attack_find_char(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackAttacker => {
                ai_action_attack_attacker(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackOwnerTarget => {
                ai_action_attack_owner_target(ai_system_parameters, ai_parameters)
            }
            AipAction::AttackNearbyEntityByStat(distance, ability_type, stat_choice) => {
                ai_action_attack_nearby_entity_by_stat(
                    ai_system_parameters,
                    ai_parameters,
                    distance,
                    ability_type,
                    stat_choice,
                )
            }
            AipAction::DoQuestTrigger(ref trigger_name) => {
                ai_action_quest_trigger(ai_system_parameters, ai_parameters, trigger_name)
            }
            AipAction::KillSelf => ai_action_kill_self(ai_system_parameters, ai_parameters),
            AipAction::NearbyAlliesAttackTarget(distance, nearby_ally_type, limit) => {
                ai_action_nearby_allies_attack_target(
                    ai_system_parameters,
                    ai_parameters,
                    distance,
                    nearby_ally_type,
                    limit,
                )
            }
            AipAction::SpawnNpc(npc_id, distance, origin, is_owner) => ai_action_spawn_npc(
                ai_system_parameters,
                ai_system_resources,
                ai_parameters,
                npc_id,
                distance,
                origin,
                is_owner,
            ),
            AipAction::TransformNpc(npc_id) => {
                ai_action_transform_npc(ai_system_parameters, ai_parameters, npc_id)
            }
            AipAction::Emote(motion_id) => {
                ai_action_use_emote(ai_system_parameters, ai_parameters, motion_id)
            }
            AipAction::UseSkill(target, skill_id, motion_id) => ai_action_use_skill(
                ai_system_parameters,
                ai_parameters,
                target,
                skill_id,
                motion_id,
            ),
            AipAction::SetMonsterSpawnState(zone, state) => {
                ai_action_set_monster_spawn_state(ai_system_parameters, ai_parameters, zone, state)
            }
            AipAction::SetVariable(variable_type, variable_id, operator, value) => {
                ai_action_set_variable(
                    ai_system_parameters,
                    ai_parameters,
                    variable_type,
                    variable_id,
                    operator,
                    value,
                )
            }
            AipAction::Message(message_type, string_id) => ai_action_message(
                ai_system_parameters,
                ai_system_resources,
                ai_parameters,
                message_type,
                string_id,
            ),
            AipAction::Say(_) => {}        // This is client side only
            AipAction::SpecialAttack => {} // This is not actually used, probably an old removed feature
            AipAction::DropRandomItem(ref items_base1000) => ai_action_drop_random_item(
                ai_system_parameters,
                ai_system_resources,
                ai_parameters,
                items_base1000,
            ),
            AipAction::GiveItemToOwner(item_base1000, quantity) => ai_action_give_item_to_owner(
                ai_system_parameters,
                ai_system_resources,
                ai_parameters,
                item_base1000,
                quantity,
            ),
            /*
            AipAction::SetPvpFlag(_, _) => {}
            */
            _ => {
                log::warn!(target: "npc_ai_unimplemented", "Unimplemented AI action: {:?}", action);
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
    log::trace!(target: "npc_ai", "Running AI trigger");
    for (index, ai_program_event) in ai_trigger.events.iter().enumerate() {
        log::trace!(target: "npc_ai", " - Event {}", index);
        if npc_ai_check_conditions(
            ai_system_parameters,
            ai_system_resources,
            ai_program_event,
            &mut ai_parameters,
        ) {
            npc_ai_do_actions(
                ai_system_parameters,
                ai_system_resources,
                ai_program_event,
                &mut ai_parameters,
            );
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
    )) = attacker_query.get_inner(entity)
    {
        Some(AiAttackerData::<'w> {
            entity,
            position: attacker_position,
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
    killer_query: Query<(&Level, &AbilityValues, Option<&Owner>, Option<&GameClient>)>,
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
                npc,
                client_entity,
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

                                    if let Ok((damage_source_level, _, damage_source_owner, _)) =
                                        killer_query.get(damage_source.entity)
                                    {
                                        // If the damage source has an owner then the owner gets the reward
                                        let (reward_xp_entity, reward_xp_entity_level) =
                                            damage_source_owner
                                                .and_then(|damage_source_owner| {
                                                    killer_query
                                                        .get(damage_source_owner.entity)
                                                        .map(|(damage_source_owner_level, ..)| {
                                                            (
                                                                damage_source_owner.entity,
                                                                damage_source_owner_level.level,
                                                            )
                                                        })
                                                        .ok()
                                                })
                                                .unwrap_or((
                                                    damage_source.entity,
                                                    damage_source_level.level,
                                                ));

                                        let reward_xp = ai_system_resources
                                            .game_data
                                            .ability_value_calculator
                                            .calculate_give_xp(
                                                reward_xp_entity_level as i32,
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
                                                reward_xp_entity,
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
                                        killer_owner,
                                        killer_game_client,
                                    )) = killer_query.get(killer_entity)
                                    {
                                        // If the killer has an owner then the owner gets the reward
                                        let (
                                            killer_level,
                                            killer_ability_values,
                                            killer_game_client,
                                        ) = killer_owner
                                            .and_then(|killer_owner| {
                                                killer_query
                                                    .get(killer_owner.entity)
                                                    .ok()
                                                    .map(|(a, b, _c, d)| (a, b, d))
                                            })
                                            .unwrap_or((
                                                killer_level,
                                                killer_ability_values,
                                                killer_game_client,
                                            ));

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
