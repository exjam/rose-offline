use std::{num::NonZeroU8, ops::RangeInclusive};

use bevy_ecs::prelude::{Commands, Entity, EventReader, EventWriter, Mut, Query, Res, ResMut};
use chrono::{Datelike, Timelike};
use log::warn;
use nalgebra::{Point2, Point3};
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::{
        formats::qsd::{
            QsdCondition, QsdConditionMonthDayTime, QsdConditionObjectVariable,
            QsdConditionOperator, QsdConditionQuestItem, QsdConditionSelectEventObject,
            QsdConditionWeekDayTime, QsdDistance, QsdEventId, QsdNpcId, QsdObjectType, QsdReward,
            QsdRewardCalculatedItem, QsdRewardMonsterSpawnState, QsdRewardObjectVariable,
            QsdRewardOperator, QsdRewardQuestAction, QsdRewardSetTeamNumberSource,
            QsdRewardSpawnMonster, QsdRewardSpawnMonsterLocation, QsdRewardTarget,
            QsdServerChannelId, QsdSkillId, QsdTeamNumber, QsdVariableId, QsdVariableType,
            QsdZoneId,
        },
        item::{EquipmentItem, Item},
        AbilityType, ItemReference, NpcId, QuestTrigger, SkillId, WorldTicks, ZoneId,
    },
    game::{
        bundles::{
            ability_values_add_value, ability_values_get_value, ability_values_set_value,
            client_entity_teleport_zone, skill_list_try_learn_skill, MonsterBundle,
        },
        components::{
            AbilityValues, ActiveQuest, BasicStats, CharacterInfo, ClientEntity, Equipment,
            EquipmentIndex, ExperiencePoints, GameClient, HealthPoints, Inventory, Level,
            ManaPoints, Money, MoveSpeed, ObjectVariables, Position, QuestState, SkillList,
            SkillPoints, SpawnOrigin, Stamina, StatPoints, Team, UnionMembership,
        },
        events::{QuestTriggerEvent, RewardXpEvent},
        messages::server::{QuestTriggerResult, ServerMessage, UpdateInventory, UpdateMoney},
        resources::{ClientEntityList, ServerTime, WorldRates, WorldTime, ZoneList},
        GameData,
    },
};

struct QuestSourceEntity<'world, 'a> {
    entity: Entity,
    game_client: Option<&'a GameClient>,
    ability_values: &'a AbilityValues,
    basic_stats: Option<&'a mut Mut<'world, BasicStats>>,
    character_info: Option<&'a mut Mut<'world, CharacterInfo>>,
    client_entity: &'a ClientEntity,
    equipment: Option<&'a Equipment>,
    experience_points: Option<&'a mut Mut<'world, ExperiencePoints>>,
    health_points: Option<&'a mut Mut<'world, HealthPoints>>,
    inventory: Option<&'a mut Mut<'world, Inventory>>,
    level: &'a Level,
    mana_points: Option<&'a mut Mut<'world, ManaPoints>>,
    move_speed: &'a MoveSpeed,
    position: &'a Position,
    quest_state: Option<&'a mut Mut<'world, QuestState>>,
    skill_list: Option<&'a mut Mut<'world, SkillList>>,
    skill_points: Option<&'a mut Mut<'world, SkillPoints>>,
    stamina: Option<&'a mut Mut<'world, Stamina>>,
    stat_points: Option<&'a mut Mut<'world, StatPoints>>,
    team: &'a mut Mut<'world, Team>,
    union_membership: Option<&'a mut Mut<'world, UnionMembership>>,
}

struct QuestParameters<'a, 'world, 'b> {
    source: &'a mut QuestSourceEntity<'world, 'b>,
    selected_event_object: Option<Entity>,
    selected_npc: Option<Entity>,
    selected_quest_index: Option<usize>,
    next_trigger_name: Option<String>,
}

struct QuestWorld<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h> {
    commands: &'a mut Commands<'b>,
    client_entity_list: &'a mut ResMut<'c, ClientEntityList>,
    game_data: &'a GameData,
    server_time: &'a ServerTime,
    world_rates: &'a WorldRates,
    world_time: &'a WorldTime,
    zone_list: &'a mut ResMut<'h, ZoneList>,
    reward_xp_events: &'a mut EventWriter<'d, RewardXpEvent>,
    object_variables_query: &'a mut Query<'e, (&'f mut ObjectVariables, &'g Position)>,
    rng: ThreadRng,
}

fn quest_condition_operator<T: PartialEq + PartialOrd>(
    operator: QsdConditionOperator,
    value_lhs: T,
    value_rhs: T,
) -> bool {
    match operator {
        QsdConditionOperator::Equals => value_lhs == value_rhs,
        QsdConditionOperator::GreaterThan => value_lhs > value_rhs,
        QsdConditionOperator::GreaterThanEqual => value_lhs >= value_rhs,
        QsdConditionOperator::LessThan => value_lhs < value_rhs,
        QsdConditionOperator::LessThanEqual => value_lhs <= value_rhs,
        QsdConditionOperator::NotEqual => value_lhs != value_rhs,
    }
}

fn quest_get_expire_time(quest_world: &mut QuestWorld, quest_id: usize) -> Option<WorldTicks> {
    quest_world
        .game_data
        .quests
        .get_quest_data(quest_id)
        .and_then(|quest_data| quest_data.time_limit)
        .map(|time_limit| quest_world.world_time.ticks + time_limit)
}

fn quest_condition_select_quest(quest_parameters: &mut QuestParameters, quest_id: usize) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_ref() {
        if let Some(quest_index) = quest_state.find_active_quest_index(quest_id) {
            quest_parameters.selected_quest_index = Some(quest_index);
            return true;
        }
    }

    false
}

fn quest_condition_quest_switch(
    quest_parameters: &mut QuestParameters,
    switch_id: usize,
    value: bool,
) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
        if let Some(switch) = (*quest_state).quest_switches.get(switch_id) {
            return *switch == value;
        }
    }

    false
}

fn quest_condition_quest_item(
    quest_parameters: &QuestParameters,
    item_reference: Option<ItemReference>,
    equipment_index: Option<EquipmentIndex>,
    required_count: u32,
    operator: QsdConditionOperator,
) -> bool {
    if let Some(equipment_index) = equipment_index {
        if let Some(equipment) = quest_parameters.source.equipment.as_ref() {
            item_reference
                == equipment
                    .get_equipment_item(equipment_index)
                    .map(|item| item.item)
        } else {
            false
        }
    } else {
        let quantity = if let Some(item_reference) = item_reference {
            if item_reference.item_type.is_quest_item() {
                // Check selected quest item
                if let (Some(quest_state), Some(selected_quest_index)) = (
                    quest_parameters.source.quest_state.as_ref(),
                    quest_parameters.selected_quest_index,
                ) {
                    quest_state
                        .get_quest(selected_quest_index)
                        .and_then(|active_quest| active_quest.find_item(item_reference))
                        .map(|quest_item| quest_item.get_quantity())
                        .unwrap_or(0)
                } else {
                    0
                }
            } else {
                // Check inventory
                if let Some(inventory) = quest_parameters.source.inventory.as_ref() {
                    inventory
                        .find_item(item_reference)
                        .and_then(|slot| inventory.get_item(slot))
                        .map(|inventory_item| inventory_item.get_quantity())
                        .unwrap_or(0)
                } else {
                    0
                }
            }
        } else {
            0
        };

        quest_condition_operator(operator, quantity, required_count)
    }
}

fn quest_condition_quest_items(
    quest_parameters: &QuestParameters,
    items: &[QsdConditionQuestItem],
) -> bool {
    for &QsdConditionQuestItem {
        item,
        equipment_index,
        required_count,
        operator,
    } in items
    {
        if !quest_condition_quest_item(
            quest_parameters,
            item,
            equipment_index,
            required_count,
            operator,
        ) {
            return false;
        }
    }

    true
}

fn quest_condition_ability_values(
    quest_parameters: &QuestParameters,
    ability_values: &[(AbilityType, QsdConditionOperator, i32)],
) -> bool {
    for &(ability_type, operator, compare_value) in ability_values {
        let current_value = ability_values_get_value(
            ability_type,
            quest_parameters.source.ability_values,
            quest_parameters.source.level,
            quest_parameters.source.move_speed,
            quest_parameters.source.team,
            quest_parameters
                .source
                .character_info
                .as_deref()
                .map(|x| &**x),
            quest_parameters
                .source
                .experience_points
                .as_deref()
                .map(|x| &**x),
            quest_parameters.source.inventory.as_deref().map(|x| &**x),
            quest_parameters
                .source
                .skill_points
                .as_deref()
                .map(|x| &**x),
            quest_parameters.source.stamina.as_deref().map(|x| &**x),
            quest_parameters.source.stat_points.as_deref().map(|x| &**x),
            quest_parameters
                .source
                .union_membership
                .as_deref()
                .map(|x| &**x),
        )
        .unwrap_or(0);

        if !quest_condition_operator(operator, current_value, compare_value) {
            return false;
        }
    }

    true
}

fn quest_condition_position(
    quest_parameters: &QuestParameters,
    zone_id: QsdZoneId,
    position: Point2<f32>,
    distance: i32,
) -> bool {
    if quest_parameters.source.position.zone_id.get() as usize != zone_id {
        return false;
    }

    (nalgebra::distance(&quest_parameters.source.position.position.xy(), &position) as i32)
        < distance
}

fn get_quest_variable(
    quest_world: &QuestWorld,
    quest_parameters: &QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
) -> Option<i32> {
    if let Some(quest_state) = &quest_parameters.source.quest_state {
        let active_quest = quest_parameters
            .selected_quest_index
            .and_then(|quest_index| quest_state.get_quest(quest_index));

        match variable_type {
            QsdVariableType::Variable => active_quest
                .and_then(|active_quest| active_quest.variables.get(variable_id))
                .map(|x| *x as i32),
            QsdVariableType::Switch => active_quest
                .and_then(|active_quest| active_quest.switches.get(variable_id))
                .map(|x| *x as i32),
            QsdVariableType::Timer => active_quest
                .and_then(|active_quest| active_quest.expire_time)
                .map(|expire_time| {
                    expire_time.0.saturating_sub(quest_world.world_time.ticks.0) as i32
                }),
            QsdVariableType::Episode => quest_state
                .episode_variables
                .get(variable_id)
                .map(|x| *x as i32),
            QsdVariableType::Job => quest_state
                .job_variables
                .get(variable_id)
                .map(|x| *x as i32),
            QsdVariableType::Planet => quest_state
                .planet_variables
                .get(variable_id)
                .map(|x| *x as i32),
            QsdVariableType::Union => quest_state
                .union_variables
                .get(variable_id)
                .map(|x| *x as i32),
        }
    } else {
        None
    }
}

fn quest_condition_quest_variable(
    quest_world: &QuestWorld,
    quest_parameters: &QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
    operator: QsdConditionOperator,
    value: i32,
) -> bool {
    if let Some(variable_value) =
        get_quest_variable(quest_world, quest_parameters, variable_type, variable_id)
    {
        quest_condition_operator(operator, variable_value, value)
    } else {
        false
    }
}

fn quest_condition_world_time(quest_world: &mut QuestWorld, range: &RangeInclusive<u32>) -> bool {
    range.contains(&quest_world.world_time.ticks.get_world_time())
}

fn quest_condition_month_day_time(
    quest_world: &mut QuestWorld,
    month_day: Option<NonZeroU8>,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &quest_world.server_time.local_time;

    if let Some(month_day) = month_day {
        if month_day.get() as u32 != local_time.day() {
            return false;
        }
    }

    let local_day_minutes = local_time.hour() as i32 + local_time.minute() as i32;
    day_minutes_range.contains(&local_day_minutes)
}

fn quest_condition_week_day_time(
    quest_world: &mut QuestWorld,
    week_day: u8,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &quest_world.server_time.local_time;

    if week_day as u32 != local_time.weekday().num_days_from_sunday() {
        return false;
    }

    let local_day_minutes = local_time.hour() as i32 + local_time.minute() as i32;
    day_minutes_range.contains(&local_day_minutes)
}

fn quest_condition_have_skill(
    quest_parameters: &QuestParameters,
    skill_id_range: &RangeInclusive<QsdSkillId>,
    have: bool,
) -> bool {
    if let Some(skill_list) = &quest_parameters.source.skill_list {
        for skill_id in skill_list.iter_skills() {
            if skill_id_range.contains(&(skill_id.get() as QsdSkillId)) {
                return have;
            }
        }
    }

    !have
}

fn quest_condition_team_number(
    quest_parameters: &QuestParameters,
    range: &RangeInclusive<QsdTeamNumber>,
) -> bool {
    range.contains(&(quest_parameters.source.team.id as QsdTeamNumber))
}

fn quest_condition_server_channel_number(
    channel_range: &RangeInclusive<QsdServerChannelId>,
) -> bool {
    // TODO: Do we need to have channel numbers?
    channel_range.contains(&1)
}

fn quest_condition_select_event_object(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    zone_id: QsdZoneId,
    event_id: QsdEventId,
    map_chunk_x: i32,
    map_chunk_y: i32,
) -> bool {
    let event_object = ZoneId::new(zone_id as u16).and_then(|zone_id| {
        quest_world
            .zone_list
            .find_event_object(zone_id, event_id as u16, map_chunk_x, map_chunk_y)
    });
    quest_parameters.selected_event_object = event_object;
    event_object.is_some()
}

fn quest_condition_select_npc(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    npc_id: QsdNpcId,
) -> bool {
    quest_parameters.selected_npc =
        NpcId::new(npc_id as u16).and_then(|npc_id| quest_world.zone_list.find_npc(npc_id));
    quest_parameters.selected_npc.is_some()
}

fn quest_condition_object_variable(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    object_type: QsdObjectType,
    variable_id: usize,
    operator: QsdConditionOperator,
    value: i32,
) -> bool {
    let entity = match object_type {
        QsdObjectType::Event => quest_parameters.selected_event_object,
        QsdObjectType::Npc => quest_parameters.selected_npc,
        _ => return false,
    };

    let variable_value = entity
        .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
        .and_then(|(object_variables, _)| object_variables.variables.get(variable_id).cloned());

    if let Some(variable_value) = variable_value {
        quest_condition_operator(operator, variable_value, value)
    } else {
        false
    }
}

fn quest_condition_object_zone_time(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    object_type: QsdObjectType,
    range: &RangeInclusive<u32>,
) -> bool {
    let entity = match object_type {
        QsdObjectType::Event => quest_parameters.selected_event_object,
        QsdObjectType::Npc => quest_parameters.selected_npc,
        QsdObjectType::Owner => Some(quest_parameters.source.entity),
    };

    let zone_data = entity
        .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
        .map(|(_, position)| position.zone_id)
        .and_then(|zone_id| quest_world.game_data.zones.get_zone(zone_id));

    let world_time = quest_world.world_time.ticks.get_world_time();
    let zone_time = if let Some(zone_data) = zone_data {
        world_time % zone_data.day_cycle
    } else {
        world_time
    };
    range.contains(&zone_time)
}

fn quest_condition_object_distance(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    object_type: QsdObjectType,
    distance: i32,
) -> bool {
    let entity = match object_type {
        QsdObjectType::Event => quest_parameters.selected_event_object,
        QsdObjectType::Npc => quest_parameters.selected_npc,
        _ => return false,
    };

    entity
        .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
        .map(|(_, position)| position)
        .filter(|position| position.zone_id == quest_parameters.source.position.zone_id)
        .map(|position| {
            nalgebra::distance(
                &position.position.xy(),
                &quest_parameters.source.position.position.xy(),
            ) as i32
        })
        .map(|object_distance| object_distance < distance)
        .unwrap_or(false)
}

fn quest_condition_compare_npc_object_variables(
    quest_world: &mut QuestWorld,
    npc_variable1: (QsdNpcId, QsdVariableId),
    operator: QsdConditionOperator,
    npc_variable2: (QsdNpcId, QsdVariableId),
) -> bool {
    let value1 = NpcId::new(npc_variable1.0 as u16)
        .and_then(|npc_id| quest_world.zone_list.find_npc(npc_id))
        .and_then(|npc_entity| quest_world.object_variables_query.get_mut(npc_entity).ok())
        .and_then(|(object_variables, _)| object_variables.variables.get(npc_variable1.1).cloned())
        .unwrap_or(0);

    let value2 = NpcId::new(npc_variable2.0 as u16)
        .and_then(|npc_id| quest_world.zone_list.find_npc(npc_id))
        .and_then(|npc_entity| quest_world.object_variables_query.get_mut(npc_entity).ok())
        .and_then(|(object_variables, _)| object_variables.variables.get(npc_variable2.1).cloned())
        .unwrap_or(0);

    quest_condition_operator(operator, value1, value2)
}

fn quest_trigger_check_conditions(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match *condition {
            QsdCondition::AbilityValue(ref ability_values) => {
                quest_condition_ability_values(quest_parameters, ability_values)
            }
            QsdCondition::SelectQuest(quest_id) => {
                quest_condition_select_quest(quest_parameters, quest_id)
            }
            QsdCondition::QuestItems(ref items) => {
                quest_condition_quest_items(quest_parameters, items)
            }
            QsdCondition::QuestSwitch(switch_id, value) => {
                quest_condition_quest_switch(quest_parameters, switch_id, value)
            }
            QsdCondition::Position(zone_id, position, distance) => {
                quest_condition_position(quest_parameters, zone_id, position, distance)
            }
            QsdCondition::QuestVariable(ref quest_variables) => {
                quest_variables.iter().all(|quest_variable| {
                    quest_condition_quest_variable(
                        quest_world,
                        quest_parameters,
                        quest_variable.variable_type,
                        quest_variable.variable_id,
                        quest_variable.operator,
                        quest_variable.value,
                    )
                })
            }
            QsdCondition::WorldTime(ref range) => quest_condition_world_time(quest_world, range),
            QsdCondition::MonthDayTime(QsdConditionMonthDayTime {
                month_day,
                ref day_minutes_range,
            }) => quest_condition_month_day_time(quest_world, month_day, day_minutes_range),
            QsdCondition::WeekDayTime(QsdConditionWeekDayTime {
                week_day,
                ref day_minutes_range,
            }) => quest_condition_week_day_time(quest_world, week_day, day_minutes_range),
            QsdCondition::HasSkill(ref skill_id_range, have) => {
                quest_condition_have_skill(quest_parameters, skill_id_range, have)
            }
            QsdCondition::TeamNumber(ref range) => {
                quest_condition_team_number(quest_parameters, range)
            }
            QsdCondition::ServerChannelNumber(ref range) => {
                quest_condition_server_channel_number(range)
            }
            QsdCondition::SelectNpc(npc_id) => {
                quest_condition_select_npc(quest_world, quest_parameters, npc_id)
            }
            QsdCondition::SelectEventObject(QsdConditionSelectEventObject {
                zone,
                ref chunk,
                event_id,
            }) => quest_condition_select_event_object(
                quest_world,
                quest_parameters,
                zone,
                event_id,
                chunk.x as i32,
                chunk.y as i32,
            ),
            QsdCondition::ObjectVariable(QsdConditionObjectVariable {
                object_type,
                variable_id,
                operator,
                value,
            }) => quest_condition_object_variable(
                quest_world,
                quest_parameters,
                object_type,
                variable_id,
                operator,
                value,
            ),
            QsdCondition::ObjectZoneTime(object_type, ref range) => {
                quest_condition_object_zone_time(quest_world, quest_parameters, object_type, range)
            }
            QsdCondition::ObjectDistance(object_type, distance) => quest_condition_object_distance(
                quest_world,
                quest_parameters,
                object_type,
                distance,
            ),
            QsdCondition::CompareNpcVariables(npc_variable1, operator, npc_variable2) => {
                quest_condition_compare_npc_object_variables(
                    quest_world,
                    npc_variable1,
                    operator,
                    npc_variable2,
                )
            }
            QsdCondition::RandomPercent(_) => {
                // Random percent is only checked on client
                true
            }
            _ => {
                warn!("Unimplemented quest condition: {:?}", condition);
                false
            } /*
              // TODO: Implement party system
              QsdCondition::Party(_) => todo!(),
              QsdCondition::PartyMemberCount(_) => todo!(),

              // TODO: Implement clan system
              QsdCondition::InClan(_) => todo!(),
              QsdCondition::ClanPosition(_, _) => todo!(),
              QsdCondition::ClanPointContribution(_, _) => todo!(),
              QsdCondition::ClanLevel(_, _) => todo!(),
              QsdCondition::ClanPoints(_, _) => todo!(),
              QsdCondition::ClanMoney(_, _) => todo!(),
              QsdCondition::ClanMemberCount(_, _) => todo!(),
              QsdCondition::HasClanSkill(_, _) => todo!(),
              */
        };

        if !result {
            return false;
        }
    }

    true
}

fn quest_reward_set_quest_switch(
    quest_parameters: &mut QuestParameters,
    switch_id: usize,
    value: bool,
) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
        if let Some(mut switch) = (*quest_state).quest_switches.get_mut(switch_id) {
            *switch = value;
            return true;
        }
    }

    false
}

fn quest_reward_calculated_experience_points(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
) -> bool {
    let reward_value = quest_world
        .game_data
        .ability_value_calculator
        .calculate_reward_value(
            reward_equation_id,
            base_reward_value,
            0,
            quest_parameters.source.level.level as i32,
            quest_parameters
                .source
                .basic_stats
                .as_ref()
                .map(|x| x.charm)
                .unwrap_or(0) as i32,
            quest_parameters
                .source
                .character_info
                .as_ref()
                .map(|x| x.fame)
                .unwrap_or(0) as i32,
            quest_world.world_rates.reward_rate,
        );

    quest_world.reward_xp_events.send(RewardXpEvent::new(
        quest_parameters.source.entity,
        reward_value as u64,
        0,
        None,
    ));

    true
}

fn quest_reward_calculated_item(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
    reward_item: ItemReference,
    reward_gem: Option<ItemReference>,
) -> bool {
    let item = if reward_item.item_type.is_stackable() {
        let reward_value = quest_world
            .game_data
            .ability_value_calculator
            .calculate_reward_value(
                reward_equation_id,
                base_reward_value,
                0,
                quest_parameters.source.level.level as i32,
                quest_parameters
                    .source
                    .basic_stats
                    .as_ref()
                    .map(|x| x.charm)
                    .unwrap_or(0) as i32,
                quest_parameters
                    .source
                    .character_info
                    .as_ref()
                    .map(|x| x.fame)
                    .unwrap_or(0) as i32,
                quest_world.world_rates.reward_rate,
            );
        if reward_value > 0 {
            Item::new(&reward_item, reward_value as u32)
        } else {
            None
        }
    } else if let Some(mut item) = EquipmentItem::new(&reward_item) {
        if let Some(gem) = reward_gem {
            if gem.item_number < 300 {
                item.is_appraised = true;
                item.has_socket = false;
                item.gem = gem.item_number as u16;
            }
        }

        if item.gem == 0 {
            let item_data = quest_world.game_data.items.get_base_item(reward_item);
            let item_rare_type = item_data.map(|data| data.rare_type).unwrap_or(0);
            let item_quality = item_data.map(|data| data.quality).unwrap_or(0);

            match item_rare_type {
                1 => {
                    item.has_socket = true;
                    item.is_appraised = true;
                }
                2 => {
                    if item_quality + 60 > quest_world.rng.gen_range(0..400) {
                        item.has_socket = true;
                        item.is_appraised = true;
                    }
                }
                _ => {}
            }
        }

        Some(Item::Equipment(item))
    } else {
        None
    };

    if let Some(item) = item {
        if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
            match inventory.try_add_item(item) {
                Ok((slot, item)) => {
                    if let Some(game_client) = quest_parameters.source.game_client {
                        game_client
                            .server_message_tx
                            .send(ServerMessage::UpdateInventory(UpdateInventory {
                                is_reward: true,
                                items: vec![(slot, Some(item.clone()))],
                            }))
                            .ok();
                    }
                }
                Err(item) => {
                    // TODO: Drop item to ground
                    warn!("Unimplemented drop unclaimed quest item {:?}", item);
                }
            }
        }
    }

    true
}

fn reset_quest_calculated_money_dup_count_var(
    selected_quest_index: Option<usize>,
    quest_state: &mut Option<&mut Mut<QuestState>>,
) -> Option<()> {
    let quest_index = selected_quest_index?;
    let quest_state = quest_state.as_mut()?;
    let active_quest = quest_state.get_quest_mut(quest_index)?;
    *active_quest.variables.last_mut()? = 0;
    Some(())
}

fn get_quest_calculated_money_dup_count_var<'a, 'world>(
    selected_quest_index: Option<usize>,
    quest_state: &'a Option<&'a mut Mut<'world, QuestState>>,
) -> Option<&'a u16> {
    let quest_index = selected_quest_index?;
    let quest_state = quest_state.as_ref()?;
    let active_quest = quest_state.get_quest(quest_index)?;
    active_quest.variables.last()
}

fn quest_reward_calculated_money(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
) -> bool {
    let dup_count_var = get_quest_calculated_money_dup_count_var(
        quest_parameters.selected_quest_index,
        &quest_parameters.source.quest_state,
    );

    let reward_value = quest_world
        .game_data
        .ability_value_calculator
        .calculate_reward_value(
            reward_equation_id,
            base_reward_value,
            dup_count_var.as_ref().map_or(0, |x| **x) as i32,
            quest_parameters.source.level.level as i32,
            quest_parameters
                .source
                .basic_stats
                .as_ref()
                .map(|x| x.charm)
                .unwrap_or(0) as i32,
            quest_parameters
                .source
                .character_info
                .as_ref()
                .map(|x| x.fame)
                .unwrap_or(0) as i32,
            quest_world.world_rates.reward_rate,
        );
    let money = Money(reward_value as i64);

    if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
        if inventory.try_add_money(money).is_ok() {
            reset_quest_calculated_money_dup_count_var(
                quest_parameters.selected_quest_index,
                &mut quest_parameters.source.quest_state,
            );

            if let Some(game_client) = quest_parameters.source.game_client {
                game_client
                    .server_message_tx
                    .send(ServerMessage::UpdateMoney(UpdateMoney {
                        is_reward: true,
                        money: inventory.money,
                    }))
                    .ok();
            }
        }
    }

    true
}

fn quest_reward_quest_action(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    action: &QsdRewardQuestAction,
) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
        match *action {
            QsdRewardQuestAction::RemoveSelected => {
                if let Some(quest_index) = quest_parameters.selected_quest_index {
                    if let Some(quest_slot) = quest_state.get_quest_slot_mut(quest_index) {
                        *quest_slot = None;
                        return true;
                    }
                }
            }
            QsdRewardQuestAction::Add(quest_id) => {
                if let Some(quest_index) = quest_state.try_add_quest(ActiveQuest::new(
                    quest_id,
                    quest_get_expire_time(quest_world, quest_id),
                )) {
                    if quest_parameters.selected_quest_index.is_none() {
                        quest_parameters.selected_quest_index = Some(quest_index);
                    }

                    return true;
                }
            }
            QsdRewardQuestAction::ChangeSelectedIdKeepData(quest_id) => {
                if let Some(quest_index) = quest_parameters.selected_quest_index {
                    if let Some(Some(active_quest)) = quest_state.get_quest_slot_mut(quest_index) {
                        active_quest.quest_id = quest_id;
                        return true;
                    }
                }
            }
            QsdRewardQuestAction::ChangeSelectedIdResetData(quest_id) => {
                if let Some(quest_index) = quest_parameters.selected_quest_index {
                    if let Some(Some(active_quest)) = quest_state.get_quest_slot_mut(quest_index) {
                        *active_quest = ActiveQuest::new(
                            quest_id,
                            quest_get_expire_time(quest_world, quest_id),
                        );
                        return true;
                    }
                }
            }
            QsdRewardQuestAction::Select(quest_id) => {
                if let Some(quest_index) = quest_state.find_active_quest_index(quest_id) {
                    quest_parameters.selected_quest_index = Some(quest_index);
                    return true;
                }
            }
        }
    }

    false
}

fn quest_reward_add_item(
    quest_parameters: &mut QuestParameters,
    item_reference: ItemReference,
    quantity: usize,
) -> bool {
    if item_reference.item_type.is_quest_item() {
        // Add to quest items
        if let (Some(quest_state), Some(selected_quest_index)) = (
            quest_parameters.source.quest_state.as_mut(),
            quest_parameters.selected_quest_index,
        ) {
            return quest_state
                .get_quest_mut(selected_quest_index)
                .and_then(|active_quest| {
                    Item::new(&item_reference, quantity as u32)
                        .and_then(|item| active_quest.try_add_item(item).ok())
                })
                .is_some();
        }
    } else {
        // Add to inventory
        if let Some(item) = Item::new(&item_reference, quantity as u32) {
            if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
                match inventory.try_add_item(item) {
                    Ok((slot, item)) => {
                        if let Some(game_client) = quest_parameters.source.game_client {
                            game_client
                                .server_message_tx
                                .send(ServerMessage::UpdateInventory(UpdateInventory {
                                    is_reward: true,
                                    items: vec![(slot, Some(item.clone()))],
                                }))
                                .ok();
                        }

                        return true;
                    }
                    Err(item) => {
                        // TODO: Drop item to ground
                        warn!("Unimplemented drop unclaimed quest item {:?}", item);
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn quest_reward_remove_item(
    quest_parameters: &mut QuestParameters,
    item_reference: ItemReference,
    quantity: usize,
) -> bool {
    if item_reference.item_type.is_quest_item() {
        // Remove from quest items
        if let (Some(quest_state), Some(selected_quest_index)) = (
            quest_parameters.source.quest_state.as_mut(),
            quest_parameters.selected_quest_index,
        ) {
            return quest_state
                .get_quest_mut(selected_quest_index)
                .and_then(|active_quest| {
                    active_quest.try_take_item(item_reference, quantity as u32)
                })
                .is_some();
        }
    } else if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
        // We do not need to send packet to client updating inventory
        return inventory
            .try_take_item(item_reference, quantity as u32)
            .is_some();
    }

    false
}

fn quest_reward_add_skill(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    skill_id: QsdSkillId,
) -> Option<()> {
    let skill_id = SkillId::new(skill_id as u16)?;

    if let Some(skill_list) = quest_parameters.source.skill_list.as_deref_mut() {
        skill_list_try_learn_skill(
            quest_world.game_data.skills.as_ref(),
            skill_id,
            skill_list,
            quest_parameters.source.skill_points.as_deref_mut(),
            quest_parameters.source.game_client,
        )
        .ok()
        .map(|_| ())
    } else {
        Some(())
    }
}

fn quest_reward_remove_skill(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    skill_id: QsdSkillId,
) -> Option<()> {
    let skill_id = SkillId::new(skill_id as u16)?;
    let skill_data = quest_world.game_data.skills.get_skill(skill_id)?;
    let skill_list = quest_parameters.source.skill_list.as_deref_mut()?;
    let (skill_slot, _) = skill_list.find_skill(skill_data)?;
    let skill_slot = skill_list.get_slot_mut(skill_slot)?;
    *skill_slot = None;
    Some(())
}

fn quest_reward_reset_basic_stats(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
) -> bool {
    if let Some(character_info) = quest_parameters.source.character_info.as_ref() {
        if let Ok(reset_basic_stats) = quest_world
            .game_data
            .character_creator
            .get_basic_stats(character_info.gender)
        {
            let mut total_stat_points = 0;
            for level in 2..=quest_parameters.source.level.level {
                total_stat_points += quest_world
                    .game_data
                    .ability_value_calculator
                    .calculate_levelup_reward_stat_points(level);
            }

            if let Some(basic_stats) = quest_parameters.source.basic_stats.as_deref_mut() {
                **basic_stats = reset_basic_stats;
            }

            if let Some(stat_points) = quest_parameters.source.stat_points.as_deref_mut() {
                stat_points.points = total_stat_points;
            }

            return true;
        }
    }

    false
}

fn quest_reward_reset_skills(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
) -> bool {
    if let Some(skill_list) = quest_parameters.source.skill_list.as_deref_mut() {
        skill_list.active.skills = Default::default();
        skill_list.passive.skills = Default::default();
        skill_list.clan.skills = Default::default();

        let mut total_skill_points = 0;
        for level in 2..=quest_parameters.source.level.level {
            total_skill_points += quest_world
                .game_data
                .ability_value_calculator
                .calculate_levelup_reward_skill_points(level);
        }

        if let Some(skill_points) = quest_parameters.source.skill_points.as_deref_mut() {
            skill_points.points = total_skill_points;
        }

        true
    } else {
        false
    }
}

fn quest_reward_teleport(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    new_zone_id: ZoneId,
    new_position: Point3<f32>,
) -> bool {
    client_entity_teleport_zone(
        quest_world.commands,
        quest_world.client_entity_list,
        quest_parameters.source.entity,
        quest_parameters.source.client_entity,
        quest_parameters.source.position,
        Position::new(new_position, new_zone_id),
        quest_parameters.source.game_client,
    );
    true
}

fn quest_reward_ability_value(
    quest_parameters: &mut QuestParameters,
    reward_operator: QsdRewardOperator,
    ability_type: AbilityType,
    value: i32,
) -> bool {
    match reward_operator {
        QsdRewardOperator::Set => ability_values_set_value(
            ability_type,
            value,
            quest_parameters.source.basic_stats.as_deref_mut(),
            quest_parameters.source.character_info.as_deref_mut(),
            quest_parameters.source.union_membership.as_deref_mut(),
            quest_parameters.source.game_client.as_deref(),
        ),
        QsdRewardOperator::Add => ability_values_add_value(
            ability_type,
            value,
            quest_parameters.source.basic_stats.as_deref_mut(),
            quest_parameters.source.inventory.as_deref_mut(),
            quest_parameters.source.skill_points.as_deref_mut(),
            quest_parameters.source.stamina.as_deref_mut(),
            quest_parameters.source.stat_points.as_deref_mut(),
            quest_parameters.source.union_membership.as_deref_mut(),
            quest_parameters.source.game_client.as_deref(),
        ),
        QsdRewardOperator::Subtract => ability_values_add_value(
            ability_type,
            -value,
            quest_parameters.source.basic_stats.as_deref_mut(),
            quest_parameters.source.inventory.as_deref_mut(),
            quest_parameters.source.skill_points.as_deref_mut(),
            quest_parameters.source.stamina.as_deref_mut(),
            quest_parameters.source.stat_points.as_deref_mut(),
            quest_parameters.source.union_membership.as_deref_mut(),
            quest_parameters.source.game_client.as_deref(),
        ),
        QsdRewardOperator::Zero => ability_values_set_value(
            ability_type,
            0,
            quest_parameters.source.basic_stats.as_deref_mut(),
            quest_parameters.source.character_info.as_deref_mut(),
            quest_parameters.source.union_membership.as_deref_mut(),
            quest_parameters.source.game_client.as_deref(),
        ),
        QsdRewardOperator::One => ability_values_set_value(
            ability_type,
            1,
            quest_parameters.source.basic_stats.as_deref_mut(),
            quest_parameters.source.character_info.as_deref_mut(),
            quest_parameters.source.union_membership.as_deref_mut(),
            quest_parameters.source.game_client.as_deref(),
        ),
    }
}

fn quest_reward_operator(operator: QsdRewardOperator, variable_value: i32, value: i32) -> i32 {
    match operator {
        QsdRewardOperator::Set => value,
        QsdRewardOperator::Add => variable_value + value,
        QsdRewardOperator::Subtract => variable_value - value,
        QsdRewardOperator::Zero => 0,
        QsdRewardOperator::One => 1,
    }
}

fn set_quest_variable(
    quest_parameters: &mut QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
    value: i32,
) {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_deref_mut() {
        let active_quest = quest_parameters
            .selected_quest_index
            .and_then(|quest_index| quest_state.get_quest_mut(quest_index));

        match variable_type {
            QsdVariableType::Variable => active_quest
                .and_then(|active_quest| active_quest.variables.get_mut(variable_id))
                .map(|x| *x = value as u16),
            QsdVariableType::Switch => active_quest
                .and_then(|active_quest| active_quest.switches.get_mut(variable_id))
                .map(|mut x| *x = value != 0),
            QsdVariableType::Episode => quest_state
                .episode_variables
                .get_mut(variable_id)
                .map(|x| *x = value as u16),
            QsdVariableType::Job => quest_state
                .job_variables
                .get_mut(variable_id)
                .map(|x| *x = value as u16),
            QsdVariableType::Planet => quest_state
                .planet_variables
                .get_mut(variable_id)
                .map(|x| *x = value as u16),
            QsdVariableType::Union => quest_state
                .union_variables
                .get_mut(variable_id)
                .map(|x| *x = value as u16),
            QsdVariableType::Timer => None, // Does nothing
        };
    }
}

fn quest_reward_quest_variable(
    quest_world: &QuestWorld,
    quest_parameters: &mut QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
    operator: QsdRewardOperator,
    value: i32,
) -> bool {
    if let Some(variable_value) =
        get_quest_variable(quest_world, quest_parameters, variable_type, variable_id)
    {
        let value = quest_reward_operator(operator, variable_value, value);
        set_quest_variable(quest_parameters, variable_type, variable_id, value);
        true
    } else {
        false
    }
}

fn quest_reward_set_health_mana_percent(
    quest_parameters: &mut QuestParameters,
    health_percent: i32,
    mana_percent: i32,
) -> bool {
    if let Some(health_points) = quest_parameters.source.health_points.as_mut() {
        health_points.hp = ((quest_parameters.source.ability_values.get_max_health()
            * health_percent)
            / 100) as u32;
    }

    if let Some(mana_points) = quest_parameters.source.mana_points.as_mut() {
        mana_points.mp =
            ((quest_parameters.source.ability_values.get_max_mana() * mana_percent) / 100) as u32;
    }

    true
}

fn quest_reward_object_variable(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    object_type: QsdObjectType,
    variable_id: usize,
    operator: QsdRewardOperator,
    value: i32,
) -> bool {
    let entity = match object_type {
        QsdObjectType::Event => quest_parameters.selected_event_object,
        QsdObjectType::Npc => quest_parameters.selected_npc,
        _ => return false,
    };

    entity
        .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
        .map(|(mut object_variables, _)| {
            if let Some(variable_value) = object_variables.variables.get_mut(variable_id) {
                *variable_value = quest_reward_operator(operator, *variable_value, value);
                true
            } else {
                false
            }
        })
        .unwrap_or(false)
}

fn quest_reward_spawn_monster(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    npc: QsdNpcId,
    count: usize,
    location: QsdRewardSpawnMonsterLocation,
    distance: QsdDistance,
    team_number: QsdTeamNumber,
) -> bool {
    if let Some(npc_id) = NpcId::new(npc as u16) {
        if let Some((spawn_zone, spawn_position)) = match location {
            QsdRewardSpawnMonsterLocation::Owner => Some((
                quest_parameters.source.position.zone_id,
                quest_parameters.source.position.position,
            )),
            QsdRewardSpawnMonsterLocation::Npc => quest_parameters
                .selected_event_object
                .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
                .map(|(_, position)| (position.zone_id, position.position)),
            QsdRewardSpawnMonsterLocation::Event => quest_parameters
                .selected_npc
                .and_then(|entity| quest_world.object_variables_query.get_mut(entity).ok())
                .map(|(_, position)| (position.zone_id, position.position)),
            QsdRewardSpawnMonsterLocation::Position(zone_id, position) => {
                ZoneId::new(zone_id as u16)
                    .map(|zone_id| (zone_id, Point3::new(position.x, position.y, 0.0)))
            }
        } {
            for _ in 0..count {
                MonsterBundle::spawn(
                    quest_world.commands,
                    quest_world.client_entity_list,
                    quest_world.game_data,
                    npc_id,
                    spawn_zone,
                    SpawnOrigin::Quest(quest_parameters.source.entity, spawn_position),
                    distance,
                    Team::new(team_number as u32),
                    None,
                    None,
                );
            }
        }
    }

    true
}

fn quest_reward_clear_all_switches(quest_parameters: &mut QuestParameters) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_deref_mut() {
        quest_state.quest_switches.set_all(false);
        true
    } else {
        false
    }
}

fn quest_reward_clear_switch_group(quest_parameters: &mut QuestParameters, group: usize) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_deref_mut() {
        for i in (32 * group)..(32 * (group + 1)) {
            if let Some(mut switch) = quest_state.quest_switches.get_mut(i) {
                *switch = false;
            }
        }
        true
    } else {
        false
    }
}

fn quest_reward_set_team_number(
    quest_parameters: &mut QuestParameters,
    source: QsdRewardSetTeamNumberSource,
) -> bool {
    let team = match source {
        QsdRewardSetTeamNumberSource::Unique => {
            Team::with_unique_id(quest_parameters.source.client_entity.id.0 as u32)
        }
        _ => {
            warn!("Unimplemented set team number source {:?}", source);
            return false;
        }
    };

    **quest_parameters.source.team = team;
    true
}

fn quest_reward_set_monster_spawn_state(
    quest_world: &mut QuestWorld,
    zone_id: QsdZoneId,
    state: QsdRewardMonsterSpawnState,
) -> bool {
    if let Some(zone_id) = ZoneId::new(zone_id as u16) {
        let enabled = match state {
            QsdRewardMonsterSpawnState::Disabled => false,
            QsdRewardMonsterSpawnState::Enabled => true,
            QsdRewardMonsterSpawnState::Toggle => {
                !quest_world.zone_list.get_monster_spawns_enabled(zone_id)
            }
        };

        quest_world
            .zone_list
            .set_monster_spawns_enabled(zone_id, enabled)
    } else {
        false
    }
}

fn quest_trigger_apply_rewards(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for reward in quest_trigger.rewards.iter() {
        let result = match *reward {
            QsdReward::Quest(ref action) => {
                quest_reward_quest_action(quest_world, quest_parameters, action)
            }
            QsdReward::AbilityValue(ref values) => {
                for &(ability_type, reward_operator, value) in values {
                    quest_reward_ability_value(
                        quest_parameters,
                        reward_operator,
                        ability_type,
                        value,
                    );
                }
                true
            }
            QsdReward::AddItem(_reward_target, item, quantity) => {
                quest_reward_add_item(quest_parameters, item, quantity)
            }
            QsdReward::RemoveItem(_reward_target, item, quantity) => {
                quest_reward_remove_item(quest_parameters, item, quantity)
            }
            QsdReward::AddSkill(skill_id) => {
                quest_reward_add_skill(quest_world, quest_parameters, skill_id).is_some()
            }
            QsdReward::RemoveSkill(skill_id) => {
                quest_reward_remove_skill(quest_world, quest_parameters, skill_id).is_some()
            }
            QsdReward::ResetBasicStats => {
                quest_reward_reset_basic_stats(quest_world, quest_parameters)
            }
            QsdReward::ResetSkills => quest_reward_reset_skills(quest_world, quest_parameters),
            QsdReward::SetQuestSwitch(switch_id, value) => {
                quest_reward_set_quest_switch(quest_parameters, switch_id, value)
            }
            QsdReward::CalculatedExperiencePoints(
                reward_target,
                reward_equation_id,
                base_reward_value,
            ) => quest_reward_calculated_experience_points(
                quest_world,
                quest_parameters,
                reward_target,
                reward_equation_id,
                base_reward_value,
            ),
            QsdReward::CalculatedItem(
                reward_target,
                QsdRewardCalculatedItem {
                    equation: reward_equation_id,
                    value: base_reward_value,
                    item,
                    gem,
                },
            ) => quest_reward_calculated_item(
                quest_world,
                quest_parameters,
                reward_target,
                reward_equation_id,
                base_reward_value,
                item,
                gem,
            ),
            QsdReward::CalculatedMoney(reward_target, reward_equation_id, base_reward_value) => {
                quest_reward_calculated_money(
                    quest_world,
                    quest_parameters,
                    reward_target,
                    reward_equation_id,
                    base_reward_value,
                )
            }
            QsdReward::CallLuaFunction(_) => {
                // CallLuaFunction is for client side only.
                true
            }
            QsdReward::Teleport(_reward_target, zone_id, ref position) => quest_reward_teleport(
                quest_world,
                quest_parameters,
                ZoneId::new(zone_id as u16).unwrap(),
                Point3::new(position.x, position.y, 0.0),
            ),
            QsdReward::Trigger(ref name) => {
                quest_parameters.next_trigger_name = Some(name.clone());
                true
            }
            QsdReward::QuestVariable(ref quest_variables) => {
                quest_variables.iter().all(|quest_variable| {
                    quest_reward_quest_variable(
                        quest_world,
                        quest_parameters,
                        quest_variable.variable_type,
                        quest_variable.variable_id,
                        quest_variable.operator,
                        quest_variable.value,
                    )
                })
            }
            QsdReward::SetHealthManaPercent(_target, health_percent, mana_percent) => {
                quest_reward_set_health_mana_percent(
                    quest_parameters,
                    health_percent as i32,
                    mana_percent as i32,
                )
            }
            QsdReward::ObjectVariable(QsdRewardObjectVariable {
                object_type,
                variable_id,
                operator,
                value,
            }) => quest_reward_object_variable(
                quest_world,
                quest_parameters,
                object_type,
                variable_id,
                operator,
                value,
            ),
            QsdReward::SpawnMonster(QsdRewardSpawnMonster {
                npc,
                count,
                location,
                distance,
                team_number,
            }) => quest_reward_spawn_monster(
                quest_world,
                quest_parameters,
                npc,
                count,
                location,
                distance,
                team_number,
            ),
            QsdReward::ClearAllSwitches => quest_reward_clear_all_switches(quest_parameters),
            QsdReward::ClearSwitchGroup(group) => {
                quest_reward_clear_switch_group(quest_parameters, group)
            }
            QsdReward::SetTeamNumber(source) => {
                quest_reward_set_team_number(quest_parameters, source)
            }
            QsdReward::SetMonsterSpawnState(zone_id, state) => {
                quest_reward_set_monster_spawn_state(quest_world, zone_id, state)
            }
            _ => {
                warn!("Unimplemented quest reward: {:?}", reward);
                false
            } /*
              QsdReward::NpcMessage(_, _) => todo!(),
              QsdReward::TriggerAfterDelayForObject(_, _, _) => todo!(),
              QsdReward::FormatAnnounceMessage(_, _) => todo!(),
              QsdReward::TriggerForZoneTeam(_, _, _) => todo!(),
              QsdReward::SetRevivePosition(_) => todo!(),


              // TODO: Implement clans
              QsdReward::ClanLevel(_, _) => todo!(),
              QsdReward::ClanMoney(_, _) => todo!(),
              QsdReward::ClanPoints(_, _) => todo!(),
              QsdReward::AddClanSkill(_) => todo!(),
              QsdReward::RemoveClanSkill(_) => todo!(),
              QsdReward::ClanPointContribution(_, _) => todo!(),
              QsdReward::TeleportNearbyClanMembers(_, _, _) => todo!(),
              */
        };

        if !result {
            return false;
        }
    }

    true
}

pub fn quest_system(
    mut commands: Commands,
    mut query: Query<(
        &ClientEntity,
        &AbilityValues,
        &Level,
        &MoveSpeed,
        &Position,
        Option<&Equipment>,
        (
            &mut Team,
            Option<&mut BasicStats>,
            Option<&mut CharacterInfo>,
            Option<&mut ExperiencePoints>,
            Option<&mut HealthPoints>,
            Option<&mut Inventory>,
            Option<&mut ManaPoints>,
            Option<&mut QuestState>,
            Option<&mut SkillList>,
            Option<&mut SkillPoints>,
            Option<&mut Stamina>,
            Option<&mut StatPoints>,
            Option<&mut UnionMembership>,
        ),
        Option<&GameClient>,
    )>,
    mut object_variables_query: Query<(&mut ObjectVariables, &Position)>,
    mut client_entity_list: ResMut<ClientEntityList>,
    game_data: Res<GameData>,
    world_rates: Res<WorldRates>,
    server_time: Res<ServerTime>,
    world_time: Res<WorldTime>,
    mut zone_list: ResMut<ZoneList>,
    mut quest_trigger_events: EventReader<QuestTriggerEvent>,
    mut reward_xp_events: EventWriter<RewardXpEvent>,
) {
    let mut quest_world = QuestWorld {
        commands: &mut commands,
        client_entity_list: &mut client_entity_list,
        game_data: &game_data,
        server_time: &server_time,
        world_rates: &world_rates,
        world_time: &world_time,
        zone_list: &mut zone_list,
        reward_xp_events: &mut reward_xp_events,
        object_variables_query: &mut object_variables_query,
        rng: rand::thread_rng(),
    };

    for &QuestTriggerEvent {
        trigger_entity,
        trigger_hash,
    } in quest_trigger_events.iter()
    {
        let mut trigger = game_data.quests.get_trigger_by_hash(trigger_hash);
        let mut success = false;

        if let Ok((
            client_entity,
            ability_values,
            level,
            move_speed,
            position,
            equipment,
            (
                mut team,
                mut basic_stats,
                mut character_info,
                mut experience_points,
                mut health_points,
                mut inventory,
                mut mana_points,
                mut quest_state,
                mut skill_list,
                mut skill_points,
                mut stamina,
                mut stat_points,
                mut union_membership,
            ),
            game_client,
        )) = query.get_mut(trigger_entity)
        {
            let mut quest_parameters = QuestParameters {
                source: &mut QuestSourceEntity {
                    entity: trigger_entity,
                    game_client,
                    ability_values,
                    basic_stats: basic_stats.as_mut(),
                    character_info: character_info.as_mut(),
                    client_entity,
                    equipment,
                    experience_points: experience_points.as_mut(),
                    health_points: health_points.as_mut(),
                    inventory: inventory.as_mut(),
                    level,
                    mana_points: mana_points.as_mut(),
                    move_speed,
                    position,
                    quest_state: quest_state.as_mut(),
                    skill_list: skill_list.as_mut(),
                    skill_points: skill_points.as_mut(),
                    stamina: stamina.as_mut(),
                    stat_points: stat_points.as_mut(),
                    team: &mut team,
                    union_membership: union_membership.as_mut(),
                },
                selected_event_object: None,
                selected_npc: None,
                selected_quest_index: None,
                next_trigger_name: None,
            };

            while trigger.is_some() {
                let quest_trigger = trigger.unwrap();

                if quest_trigger_check_conditions(
                    &mut quest_world,
                    &mut quest_parameters,
                    quest_trigger,
                ) && quest_trigger_apply_rewards(
                    &mut quest_world,
                    &mut quest_parameters,
                    quest_trigger,
                ) {
                    success = true;
                    break;
                }

                if quest_parameters.next_trigger_name.is_some() {
                    trigger = quest_parameters
                        .next_trigger_name
                        .take()
                        .and_then(|name| game_data.quests.get_trigger_by_name(&name));
                } else {
                    trigger = trigger
                        .unwrap()
                        .next_trigger_name
                        .as_ref()
                        .and_then(|name| game_data.quests.get_trigger_by_name(name));
                }
            }

            if let Some(game_client) = game_client {
                game_client
                    .server_message_tx
                    .send(ServerMessage::QuestTriggerResult(QuestTriggerResult {
                        success,
                        trigger_hash,
                    }))
                    .ok();
            }
        }
    }
}
