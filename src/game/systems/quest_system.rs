use bevy::ecs::{
    prelude::{Commands, Entity, EventReader, EventWriter, Mut, Query, Res, ResMut},
    query::WorldQuery,
    system::SystemParam,
};
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use chrono::{Datelike, Timelike};
use log::warn;
use rand::Rng;
use std::{marker::PhantomData, num::NonZeroU8, ops::RangeInclusive};

use rose_data::{EquipmentItem, Item, NpcId, QuestTrigger, SkillId, WorldTicks, ZoneId};
use rose_file_readers::{
    QsdAbilityType, QsdCondition, QsdConditionCheckParty, QsdConditionMonthDayTime,
    QsdConditionObjectVariable, QsdConditionOperator, QsdConditionQuestItem,
    QsdConditionSelectEventObject, QsdConditionWeekDayTime, QsdDistance, QsdEquipmentIndex,
    QsdEventId, QsdItemBase1000, QsdNpcId, QsdObjectType, QsdReward, QsdRewardCalculatedItem,
    QsdRewardMonsterSpawnState, QsdRewardNpcMessageType, QsdRewardObjectVariable,
    QsdRewardOperator, QsdRewardQuestAction, QsdRewardSetTeamNumberSource, QsdRewardSpawnMonster,
    QsdRewardSpawnMonsterLocation, QsdRewardTarget, QsdServerChannelId, QsdSkillId, QsdTeamNumber,
    QsdVariableId, QsdVariableType, QsdZoneId,
};

use crate::game::{
    bundles::{
        ability_values_add_value, ability_values_get_value, ability_values_set_value,
        client_entity_teleport_zone, skill_list_try_learn_skill, MonsterBundle,
    },
    components::{
        AbilityValues, ActiveQuest, BasicStats, CharacterInfo, ClientEntity, ClientEntitySector,
        Equipment, ExperiencePoints, GameClient, HealthPoints, Inventory, Level, ManaPoints, Money,
        MoveSpeed, Npc, ObjectVariables, Party, PartyMembership, Position, QuestState, SkillList,
        SkillPoints, SpawnOrigin, Stamina, StatPoints, Team, UnionMembership,
    },
    events::{QuestTriggerEvent, RewardItemEvent, RewardXpEvent},
    messages::server::{AnnounceChat, LocalChat, QuestTriggerResult, ServerMessage, ShoutChat},
    resources::{ClientEntityList, ServerMessages, ServerTime, WorldRates, WorldTime, ZoneList},
    GameData,
};

#[derive(SystemParam)]
pub struct QuestSystemParameters<'w, 's> {
    commands: Commands<'w, 's>,
    client_entity_list: ResMut<'w, ClientEntityList>,
    server_messages: ResMut<'w, ServerMessages>,
    zone_list: ResMut<'w, ZoneList>,
    reward_item_events: EventWriter<'w, 's, RewardItemEvent>,
    reward_xp_events: EventWriter<'w, 's, RewardXpEvent>,
    object_variables_query: Query<'w, 's, (&'static mut ObjectVariables, &'static Position)>,
    party_query: Query<'w, 's, &'static Party>,
}

#[derive(SystemParam)]
pub struct QuestSystemResources<'w, 's> {
    game_data: Res<'w, GameData>,
    server_time: Res<'w, ServerTime>,
    world_rates: Res<'w, WorldRates>,
    world_time: Res<'w, WorldTime>,

    #[system_param(ignore)]
    _secret: PhantomData<&'s ()>,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct QuestSourceEntityQuery<'w> {
    entity: Entity,
    ability_values: &'w AbilityValues,
    basic_stats: Option<&'w mut BasicStats>,
    character_info: Option<&'w mut CharacterInfo>,
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    equipment: Option<&'w Equipment>,
    experience_points: Option<&'w mut ExperiencePoints>,
    game_client: Option<&'w GameClient>,
    health_points: Option<&'w mut HealthPoints>,
    inventory: Option<&'w mut Inventory>,
    level: &'w Level,
    mana_points: Option<&'w mut ManaPoints>,
    move_speed: &'w MoveSpeed,
    npc: Option<&'w Npc>,
    party_membership: Option<&'w PartyMembership>,
    position: &'w Position,
    quest_state: Option<&'w mut QuestState>,
    skill_list: Option<&'w mut SkillList>,
    skill_points: Option<&'w mut SkillPoints>,
    stamina: Option<&'w mut Stamina>,
    stat_points: Option<&'w mut StatPoints>,
    team: &'w mut Team,
    union_membership: Option<&'w mut UnionMembership>,
}

struct QuestParameters<'a, 'b, 'w> {
    source: &'a mut QuestSourceEntityQueryItem<'b, 'w>,
    selected_event_object: Option<Entity>,
    selected_npc: Option<Entity>,
    selected_quest_index: Option<usize>,
    next_trigger_name: Option<String>,
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

fn quest_get_expire_time(
    quest_system_resources: &QuestSystemResources,
    quest_id: usize,
) -> Option<WorldTicks> {
    quest_system_resources
        .game_data
        .quests
        .get_quest_data(quest_id)
        .and_then(|quest_data| quest_data.time_limit)
        .map(|time_limit| quest_system_resources.world_time.ticks + time_limit)
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
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &QuestParameters,
    item_base1000: Option<QsdItemBase1000>,
    equipment_index: Option<QsdEquipmentIndex>,
    required_count: u32,
    operator: QsdConditionOperator,
) -> bool {
    let item_reference = item_base1000.and_then(|item_base1000| {
        quest_system_resources
            .game_data
            .data_decoder
            .decode_item_base1000(item_base1000.get() as usize)
    });

    let equipment_index = equipment_index.and_then(|equipment_index| {
        quest_system_resources
            .game_data
            .data_decoder
            .decode_equipment_index(equipment_index.get())
    });

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
    quest_system_resources: &QuestSystemResources,
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
            quest_system_resources,
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
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &QuestParameters,
    ability_values: &[(QsdAbilityType, QsdConditionOperator, i32)],
) -> bool {
    for &(ability_type, operator, compare_value) in ability_values {
        let ability_type = quest_system_resources
            .game_data
            .data_decoder
            .decode_ability_type(ability_type.get());
        if ability_type.is_none() {
            return false;
        }

        let current_value = ability_values_get_value(
            ability_type.unwrap(),
            quest_parameters.source.ability_values,
            quest_parameters.source.level,
            quest_parameters.source.move_speed,
            quest_parameters.source.team.as_ref(),
            quest_parameters.source.character_info.as_deref(),
            quest_parameters.source.experience_points.as_deref(),
            quest_parameters.source.inventory.as_deref(),
            quest_parameters.source.skill_points.as_deref(),
            quest_parameters.source.stamina.as_deref(),
            quest_parameters.source.stat_points.as_deref(),
            quest_parameters.source.union_membership.as_deref(),
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
    position: Vec2,
    distance: i32,
) -> bool {
    if quest_parameters.source.position.zone_id.get() as usize != zone_id {
        return false;
    }

    quest_parameters
        .source
        .position
        .position
        .xy()
        .distance_squared(position)
        < (distance as f32 * distance as f32)
}

fn get_quest_variable(
    quest_system_resources: &QuestSystemResources,
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
                    expire_time
                        .0
                        .saturating_sub(quest_system_resources.world_time.ticks.0)
                        as i32
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
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
    operator: QsdConditionOperator,
    value: i32,
) -> bool {
    if let Some(variable_value) = get_quest_variable(
        quest_system_resources,
        quest_parameters,
        variable_type,
        variable_id,
    ) {
        quest_condition_operator(operator, variable_value, value)
    } else {
        false
    }
}

fn quest_condition_world_time(
    quest_system_resources: &QuestSystemResources,
    range: &RangeInclusive<u32>,
) -> bool {
    range.contains(&quest_system_resources.world_time.ticks.get_world_time())
}

fn quest_condition_month_day_time(
    quest_system_resources: &QuestSystemResources,
    month_day: Option<NonZeroU8>,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &quest_system_resources.server_time.local_time;

    if let Some(month_day) = month_day {
        if month_day.get() as u32 != local_time.day() {
            return false;
        }
    }

    let local_day_minutes = local_time.hour() as i32 + local_time.minute() as i32;
    day_minutes_range.contains(&local_day_minutes)
}

fn quest_condition_week_day_time(
    quest_system_resources: &QuestSystemResources,
    week_day: u8,
    day_minutes_range: &RangeInclusive<i32>,
) -> bool {
    let local_time = &quest_system_resources.server_time.local_time;

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
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &mut QuestParameters,
    zone_id: QsdZoneId,
    event_id: QsdEventId,
    map_chunk_x: i32,
    map_chunk_y: i32,
) -> bool {
    let event_object = ZoneId::new(zone_id as u16).and_then(|zone_id| {
        quest_system_parameters.zone_list.find_event_object(
            zone_id,
            event_id as u16,
            map_chunk_x,
            map_chunk_y,
        )
    });
    quest_parameters.selected_event_object = event_object;
    event_object.is_some()
}

fn quest_condition_select_npc(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &mut QuestParameters,
    npc_id: QsdNpcId,
) -> bool {
    quest_parameters.selected_npc = NpcId::new(npc_id as u16)
        .and_then(|npc_id| quest_system_parameters.zone_list.find_npc(npc_id));
    quest_parameters.selected_npc.is_some()
}

fn quest_condition_object_variable(
    quest_system_parameters: &mut QuestSystemParameters,
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
        .and_then(|entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(entity)
                .ok()
        })
        .and_then(|(object_variables, _)| object_variables.variables.get(variable_id).cloned());

    if let Some(variable_value) = variable_value {
        quest_condition_operator(operator, variable_value, value)
    } else {
        false
    }
}

fn quest_condition_object_zone_time(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
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
        .and_then(|entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(entity)
                .ok()
        })
        .map(|(_, position)| position.zone_id)
        .and_then(|zone_id| quest_system_resources.game_data.zones.get_zone(zone_id));

    let world_time = quest_system_resources.world_time.ticks.get_world_time();
    let zone_time = if let Some(zone_data) = zone_data {
        world_time % zone_data.day_cycle
    } else {
        world_time
    };
    range.contains(&zone_time)
}

fn quest_condition_object_distance(
    quest_system_parameters: &mut QuestSystemParameters,
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
        .and_then(|entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(entity)
                .ok()
        })
        .map(|(_, position)| position)
        .filter(|position| position.zone_id == quest_parameters.source.position.zone_id)
        .map(|position| {
            quest_parameters
                .source
                .position
                .position
                .xy()
                .distance(position.position.xy()) as i32
        })
        .map(|object_distance| object_distance < distance)
        .unwrap_or(false)
}

fn quest_condition_compare_npc_object_variables(
    quest_system_parameters: &mut QuestSystemParameters,
    npc_variable1: (QsdNpcId, QsdVariableId),
    operator: QsdConditionOperator,
    npc_variable2: (QsdNpcId, QsdVariableId),
) -> bool {
    let value1 = NpcId::new(npc_variable1.0 as u16)
        .and_then(|npc_id| quest_system_parameters.zone_list.find_npc(npc_id))
        .and_then(|npc_entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(npc_entity)
                .ok()
        })
        .and_then(|(object_variables, _)| object_variables.variables.get(npc_variable1.1).cloned())
        .unwrap_or(0);

    let value2 = NpcId::new(npc_variable2.0 as u16)
        .and_then(|npc_id| quest_system_parameters.zone_list.find_npc(npc_id))
        .and_then(|npc_entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(npc_entity)
                .ok()
        })
        .and_then(|(object_variables, _)| object_variables.variables.get(npc_variable2.1).cloned())
        .unwrap_or(0);

    quest_condition_operator(operator, value1, value2)
}

fn quest_condition_check_party(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &QuestParameters,
    is_leader: bool,
    level_operator: QsdConditionOperator,
    level: i32,
) -> bool {
    if let Some(&PartyMembership::Member(party_entity)) = quest_parameters.source.party_membership {
        if let Ok(party) = quest_system_parameters.party_query.get(party_entity) {
            if is_leader && party.owner != quest_parameters.source.entity {
                return false;
            }

            return quest_condition_operator(level_operator, party.level, level);
        }
    }

    false
}

fn quest_condition_check_party_member_count(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &QuestParameters,
    range: &RangeInclusive<usize>,
) -> bool {
    if let Some(&PartyMembership::Member(party_entity)) = quest_parameters.source.party_membership {
        if let Ok(party) = quest_system_parameters.party_query.get(party_entity) {
            return range.contains(&party.members.len());
        }
    }

    false
}

fn quest_trigger_check_conditions(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match *condition {
            QsdCondition::AbilityValue(ref ability_values) => quest_condition_ability_values(
                quest_system_resources,
                quest_parameters,
                ability_values,
            ),
            QsdCondition::SelectQuest(quest_id) => {
                quest_condition_select_quest(quest_parameters, quest_id)
            }
            QsdCondition::QuestItems(ref items) => {
                quest_condition_quest_items(quest_system_resources, quest_parameters, items)
            }
            QsdCondition::QuestSwitch(switch_id, value) => {
                quest_condition_quest_switch(quest_parameters, switch_id, value)
            }
            QsdCondition::Position(zone_id, ref position, distance) => quest_condition_position(
                quest_parameters,
                zone_id,
                Vec2::new(position.x, position.y),
                distance,
            ),
            QsdCondition::QuestVariable(ref quest_variables) => {
                quest_variables.iter().all(|quest_variable| {
                    quest_condition_quest_variable(
                        quest_system_resources,
                        quest_parameters,
                        quest_variable.variable_type,
                        quest_variable.variable_id,
                        quest_variable.operator,
                        quest_variable.value,
                    )
                })
            }
            QsdCondition::WorldTime(ref range) => {
                quest_condition_world_time(quest_system_resources, range)
            }
            QsdCondition::MonthDayTime(QsdConditionMonthDayTime {
                month_day,
                ref day_minutes_range,
            }) => {
                quest_condition_month_day_time(quest_system_resources, month_day, day_minutes_range)
            }
            QsdCondition::WeekDayTime(QsdConditionWeekDayTime {
                week_day,
                ref day_minutes_range,
            }) => {
                quest_condition_week_day_time(quest_system_resources, week_day, day_minutes_range)
            }
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
                quest_condition_select_npc(quest_system_parameters, quest_parameters, npc_id)
            }
            QsdCondition::SelectEventObject(QsdConditionSelectEventObject {
                zone,
                ref chunk,
                event_id,
            }) => quest_condition_select_event_object(
                quest_system_parameters,
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
                quest_system_parameters,
                quest_parameters,
                object_type,
                variable_id,
                operator,
                value,
            ),
            QsdCondition::ObjectZoneTime(object_type, ref range) => {
                quest_condition_object_zone_time(
                    quest_system_parameters,
                    quest_system_resources,
                    quest_parameters,
                    object_type,
                    range,
                )
            }
            QsdCondition::ObjectDistance(object_type, distance) => quest_condition_object_distance(
                quest_system_parameters,
                quest_parameters,
                object_type,
                distance,
            ),
            QsdCondition::CompareNpcVariables(npc_variable1, operator, npc_variable2) => {
                quest_condition_compare_npc_object_variables(
                    quest_system_parameters,
                    npc_variable1,
                    operator,
                    npc_variable2,
                )
            }
            QsdCondition::Party(QsdConditionCheckParty {
                is_leader,
                level_operator,
                level,
            }) => quest_condition_check_party(
                quest_system_parameters,
                quest_parameters,
                is_leader,
                level_operator,
                level,
            ),
            QsdCondition::PartyMemberCount(ref range) => quest_condition_check_party_member_count(
                quest_system_parameters,
                quest_parameters,
                range,
            ),
            QsdCondition::RandomPercent(_) => {
                // Random percent is only checked on client
                true
            }
            _ => {
                warn!("Unimplemented quest condition: {:?}", condition);
                false
            } /*
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
            log::trace!(target: "quest", "Condition Failed {:?}", condition);
            return false;
        } else {
            log::trace!(target: "quest", "Condition Success {:?}", condition);
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
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
) -> bool {
    let reward_value = quest_system_resources
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
            quest_system_resources.world_rates.reward_rate,
        );

    quest_system_parameters
        .reward_xp_events
        .send(RewardXpEvent::new(
            quest_parameters.source.entity,
            reward_value as u64,
            0,
            None,
        ));

    true
}

fn quest_reward_calculated_item(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
    reward_item_base1000: QsdItemBase1000,
    reward_gem_base1000: Option<QsdItemBase1000>,
) -> bool {
    let reward_item = quest_system_resources
        .game_data
        .data_decoder
        .decode_item_base1000(reward_item_base1000.get());
    if reward_item.is_none() {
        return false;
    }
    let reward_item = reward_item.unwrap();

    let reward_item_data = quest_system_resources
        .game_data
        .items
        .get_base_item(reward_item);
    if reward_item_data.is_none() {
        return false;
    }
    let reward_item_data = reward_item_data.unwrap();

    let reward_gem = reward_gem_base1000.and_then(|item_base1000| {
        quest_system_resources
            .game_data
            .data_decoder
            .decode_item_base1000(item_base1000.get())
    });

    let item = if reward_item.item_type.is_stackable_item() {
        let reward_quantity = quest_system_resources
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
                quest_system_resources.world_rates.reward_rate,
            );
        if reward_quantity > 0 {
            Item::from_item_data(reward_item_data, reward_quantity as u32)
        } else {
            None
        }
    } else if let Some(mut item) = EquipmentItem::new(reward_item, reward_item_data.durability) {
        if let Some(gem) = reward_gem {
            if gem.item_number < 300 {
                item.is_appraised = true;
                item.has_socket = false;
                item.gem = gem.item_number as u16;
            }
        }

        if item.gem == 0 {
            let item_data = quest_system_resources
                .game_data
                .items
                .get_base_item(reward_item);
            let item_rare_type = item_data.map(|data| data.rare_type).unwrap_or(0);
            let item_quality = item_data.map(|data| data.quality).unwrap_or(0);

            match item_rare_type {
                1 => {
                    item.has_socket = true;
                    item.is_appraised = true;
                }
                2 => {
                    if item_quality + 60 > rand::thread_rng().gen_range(0..400) {
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
        quest_system_parameters
            .reward_item_events
            .send(RewardItemEvent::new(
                quest_parameters.source.entity,
                item,
                true,
            ));
    }

    true
}

fn reset_quest_calculated_money_dup_count_var(
    selected_quest_index: Option<usize>,
    quest_state: Option<&mut Mut<QuestState>>,
) -> Option<()> {
    let quest_index = selected_quest_index?;
    let quest_state = quest_state?;
    let active_quest = quest_state.get_quest_mut(quest_index)?;
    *active_quest.variables.last_mut()? = 0;
    Some(())
}

fn get_quest_calculated_money_dup_count_var(
    selected_quest_index: Option<usize>,
    quest_state: Option<&QuestState>,
) -> Option<&u16> {
    let quest_index = selected_quest_index?;
    let active_quest = quest_state?.get_quest(quest_index)?;
    active_quest.variables.last()
}

fn quest_reward_calculated_money(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    _reward_target: QsdRewardTarget,
    reward_equation_id: usize,
    base_reward_value: i32,
) -> bool {
    let dup_count_var = get_quest_calculated_money_dup_count_var(
        quest_parameters.selected_quest_index,
        quest_parameters.source.quest_state.as_deref(),
    );

    let reward_value = quest_system_resources
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
            quest_system_resources.world_rates.reward_rate,
        );
    let money = Money(reward_value as i64);

    if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
        if inventory.try_add_money(money).is_ok() {
            reset_quest_calculated_money_dup_count_var(
                quest_parameters.selected_quest_index,
                quest_parameters.source.quest_state.as_mut(),
            );

            if let Some(game_client) = quest_parameters.source.game_client {
                game_client
                    .server_message_tx
                    .send(ServerMessage::RewardMoney(inventory.money))
                    .ok();
            }
        }
    }

    true
}

fn quest_reward_quest_action(
    quest_system_resources: &QuestSystemResources,
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
                    quest_get_expire_time(quest_system_resources, quest_id),
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
                            quest_get_expire_time(quest_system_resources, quest_id),
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
    quest_system_resources: &QuestSystemResources,
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &mut QuestParameters,
    item_base1000: QsdItemBase1000,
    quantity: usize,
) -> bool {
    let item_reference = quest_system_resources
        .game_data
        .data_decoder
        .decode_item_base1000(item_base1000.get());
    if item_reference.is_none() {
        return false;
    }
    let item_reference = item_reference.unwrap();

    let item_data = quest_system_resources
        .game_data
        .items
        .get_base_item(item_reference);
    if item_data.is_none() {
        return false;
    }
    let item_data = item_data.unwrap();

    if item_reference.item_type.is_quest_item() {
        // Add to quest items
        if let (Some(quest_state), Some(selected_quest_index)) = (
            quest_parameters.source.quest_state.as_mut(),
            quest_parameters.selected_quest_index,
        ) {
            return quest_state
                .get_quest_mut(selected_quest_index)
                .and_then(|active_quest| {
                    Item::from_item_data(item_data, quantity as u32)
                        .and_then(|item| active_quest.try_add_item(item).ok())
                })
                .is_some();
        }
    } else {
        // Add to inventory
        if let Some(item) = Item::from_item_data(item_data, quantity as u32) {
            quest_system_parameters
                .reward_item_events
                .send(RewardItemEvent::new(
                    quest_parameters.source.entity,
                    item,
                    true,
                ));
            return true;
        }
    }

    false
}

fn quest_reward_remove_item(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    item_base1000: QsdItemBase1000,
    quantity: usize,
) -> bool {
    let item_reference = quest_system_resources
        .game_data
        .data_decoder
        .decode_item_base1000(item_base1000.get());
    if item_reference.is_none() {
        return false;
    }
    let item_reference = item_reference.unwrap();

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
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    skill_id: QsdSkillId,
) -> Option<()> {
    let skill_id = SkillId::new(skill_id as u16)?;

    if let Some(skill_list) = quest_parameters.source.skill_list.as_mut() {
        skill_list_try_learn_skill(
            quest_system_resources.game_data.skills.as_ref(),
            skill_id,
            skill_list,
            quest_parameters.source.skill_points.as_mut(),
            quest_parameters.source.game_client,
        )
        .ok()
        .map(|_| ())
    } else {
        Some(())
    }
}

fn quest_reward_remove_skill(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    skill_id: QsdSkillId,
) -> Option<()> {
    let skill_id = SkillId::new(skill_id as u16)?;
    let skill_data = quest_system_resources
        .game_data
        .skills
        .get_skill(skill_id)?;
    let skill_list = quest_parameters.source.skill_list.as_mut()?;
    let (skill_slot, _) = skill_list.find_skill(skill_data)?;
    let skill_slot = skill_list.get_slot_mut(skill_slot)?;
    *skill_slot = None;
    Some(())
}

fn quest_reward_reset_basic_stats(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
) -> bool {
    if let Some(character_info) = quest_parameters.source.character_info.as_ref() {
        if let Ok(reset_basic_stats) = quest_system_resources
            .game_data
            .character_creator
            .get_basic_stats(character_info.gender)
        {
            let mut total_stat_points = 0;
            for level in 2..=quest_parameters.source.level.level {
                total_stat_points += quest_system_resources
                    .game_data
                    .ability_value_calculator
                    .calculate_levelup_reward_stat_points(level);
            }

            if let Some(basic_stats) = quest_parameters.source.basic_stats.as_mut() {
                **basic_stats = reset_basic_stats;
            }

            if let Some(stat_points) = quest_parameters.source.stat_points.as_mut() {
                stat_points.points = total_stat_points;
            }

            return true;
        }
    }

    false
}

fn quest_reward_reset_skills(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
) -> bool {
    if let Some(skill_list) = quest_parameters.source.skill_list.as_mut() {
        skill_list.active.skills = Default::default();
        skill_list.passive.skills = Default::default();
        skill_list.clan.skills = Default::default();

        let mut total_skill_points = 0;
        for level in 2..=quest_parameters.source.level.level {
            total_skill_points += quest_system_resources
                .game_data
                .ability_value_calculator
                .calculate_levelup_reward_skill_points(level);
        }

        if let Some(skill_points) = quest_parameters.source.skill_points.as_mut() {
            skill_points.points = total_skill_points;
        }

        true
    } else {
        false
    }
}

fn quest_reward_teleport(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_parameters: &mut QuestParameters,
    new_zone_id: ZoneId,
    new_position: Vec3,
) -> bool {
    client_entity_teleport_zone(
        &mut quest_system_parameters.commands,
        &mut quest_system_parameters.client_entity_list,
        quest_parameters.source.entity,
        quest_parameters.source.client_entity,
        quest_parameters.source.client_entity_sector,
        quest_parameters.source.position,
        Position::new(new_position, new_zone_id),
        quest_parameters.source.game_client,
    );
    true
}

fn quest_reward_ability_value(
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    reward_operator: QsdRewardOperator,
    ability_type: QsdAbilityType,
    value: i32,
) -> bool {
    let ability_type = quest_system_resources
        .game_data
        .data_decoder
        .decode_ability_type(ability_type.get());
    if ability_type.is_none() {
        return false;
    }

    match reward_operator {
        QsdRewardOperator::Set => ability_values_set_value(
            ability_type.unwrap(),
            value,
            quest_parameters.source.basic_stats.as_mut(),
            quest_parameters.source.character_info.as_mut(),
            quest_parameters.source.union_membership.as_mut(),
            quest_parameters.source.game_client,
        ),
        QsdRewardOperator::Add => ability_values_add_value(
            ability_type.unwrap(),
            value,
            quest_parameters.source.basic_stats.as_mut(),
            quest_parameters.source.inventory.as_mut(),
            quest_parameters.source.skill_points.as_mut(),
            quest_parameters.source.stamina.as_mut(),
            quest_parameters.source.stat_points.as_mut(),
            quest_parameters.source.union_membership.as_mut(),
            quest_parameters.source.game_client,
        ),
        QsdRewardOperator::Subtract => ability_values_add_value(
            ability_type.unwrap(),
            -value,
            quest_parameters.source.basic_stats.as_mut(),
            quest_parameters.source.inventory.as_mut(),
            quest_parameters.source.skill_points.as_mut(),
            quest_parameters.source.stamina.as_mut(),
            quest_parameters.source.stat_points.as_mut(),
            quest_parameters.source.union_membership.as_mut(),
            quest_parameters.source.game_client,
        ),
        QsdRewardOperator::Zero => ability_values_set_value(
            ability_type.unwrap(),
            0,
            quest_parameters.source.basic_stats.as_mut(),
            quest_parameters.source.character_info.as_mut(),
            quest_parameters.source.union_membership.as_mut(),
            quest_parameters.source.game_client,
        ),
        QsdRewardOperator::One => ability_values_set_value(
            ability_type.unwrap(),
            1,
            quest_parameters.source.basic_stats.as_mut(),
            quest_parameters.source.character_info.as_mut(),
            quest_parameters.source.union_membership.as_mut(),
            quest_parameters.source.game_client,
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
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
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
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    variable_type: QsdVariableType,
    variable_id: usize,
    operator: QsdRewardOperator,
    value: i32,
) -> bool {
    if let Some(variable_value) = get_quest_variable(
        quest_system_resources,
        quest_parameters,
        variable_type,
        variable_id,
    ) {
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
        health_points.hp =
            (quest_parameters.source.ability_values.get_max_health() * health_percent) / 100;
    }

    if let Some(mana_points) = quest_parameters.source.mana_points.as_mut() {
        mana_points.mp =
            (quest_parameters.source.ability_values.get_max_mana() * mana_percent) / 100;
    }

    true
}

fn quest_reward_object_variable(
    quest_system_parameters: &mut QuestSystemParameters,
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
        .and_then(|entity| {
            quest_system_parameters
                .object_variables_query
                .get_mut(entity)
                .ok()
        })
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
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
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
                .and_then(|entity| {
                    quest_system_parameters
                        .object_variables_query
                        .get_mut(entity)
                        .ok()
                })
                .map(|(_, position)| (position.zone_id, position.position)),
            QsdRewardSpawnMonsterLocation::Event => quest_parameters
                .selected_npc
                .and_then(|entity| {
                    quest_system_parameters
                        .object_variables_query
                        .get_mut(entity)
                        .ok()
                })
                .map(|(_, position)| (position.zone_id, position.position)),
            QsdRewardSpawnMonsterLocation::Position(zone_id, position) => {
                ZoneId::new(zone_id as u16)
                    .map(|zone_id| (zone_id, Vec3::new(position.x, position.y, 0.0)))
            }
        } {
            for _ in 0..count {
                MonsterBundle::spawn(
                    &mut quest_system_parameters.commands,
                    &mut quest_system_parameters.client_entity_list,
                    &quest_system_resources.game_data,
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
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
        quest_state.quest_switches.fill(false);
        true
    } else {
        false
    }
}

fn quest_reward_clear_switch_group(quest_parameters: &mut QuestParameters, group: usize) -> bool {
    if let Some(quest_state) = quest_parameters.source.quest_state.as_mut() {
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

    *quest_parameters.source.team = team;
    true
}

fn quest_reward_set_monster_spawn_state(
    quest_system_parameters: &mut QuestSystemParameters,
    zone_id: QsdZoneId,
    state: QsdRewardMonsterSpawnState,
) -> bool {
    if let Some(zone_id) = ZoneId::new(zone_id as u16) {
        let enabled = match state {
            QsdRewardMonsterSpawnState::Disabled => false,
            QsdRewardMonsterSpawnState::Enabled => true,
            QsdRewardMonsterSpawnState::Toggle => !quest_system_parameters
                .zone_list
                .get_monster_spawns_enabled(zone_id),
        };

        quest_system_parameters
            .zone_list
            .set_monster_spawns_enabled(zone_id, enabled)
    } else {
        false
    }
}

fn quest_reward_npc_message(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    message_type: QsdRewardNpcMessageType,
    string_id: usize,
) -> bool {
    if let Some(message) = quest_system_resources
        .game_data
        .quests
        .get_quest_string(string_id as u16)
    {
        let name = if let Some(character_info) = quest_parameters.source.character_info.as_ref() {
            character_info.name.clone()
        } else if let Some(npc) = quest_parameters.source.npc.as_ref() {
            if let Some(npc_data) = quest_system_resources.game_data.npcs.get_npc(npc.id) {
                npc_data.name.clone()
            } else {
                return false;
            }
        } else {
            return false;
        };

        match message_type {
            QsdRewardNpcMessageType::Chat => {
                quest_system_parameters.server_messages.send_entity_message(
                    quest_parameters.source.client_entity,
                    ServerMessage::LocalChat(LocalChat {
                        entity_id: quest_parameters.source.client_entity.id,
                        text: message.clone(),
                    }),
                );
            }
            QsdRewardNpcMessageType::Shout => {
                // TODO: A shout message actually goes to adjacent 3 sectors rather than full zone
                quest_system_parameters.server_messages.send_zone_message(
                    quest_parameters.source.position.zone_id,
                    ServerMessage::ShoutChat(ShoutChat {
                        name,
                        text: message.clone(),
                    }),
                );
            }
            QsdRewardNpcMessageType::Announce => {
                quest_system_parameters.server_messages.send_zone_message(
                    quest_parameters.source.position.zone_id,
                    ServerMessage::AnnounceChat(AnnounceChat {
                        name: Some(name),
                        text: message.clone(),
                    }),
                );
            }
        };
    }

    true
}

fn quest_trigger_apply_rewards(
    quest_system_parameters: &mut QuestSystemParameters,
    quest_system_resources: &QuestSystemResources,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for reward in quest_trigger.rewards.iter() {
        let result = match *reward {
            QsdReward::Quest(ref action) => {
                quest_reward_quest_action(quest_system_resources, quest_parameters, action)
            }
            QsdReward::AbilityValue(ref values) => {
                for &(ability_type, reward_operator, value) in values {
                    quest_reward_ability_value(
                        quest_system_resources,
                        quest_parameters,
                        reward_operator,
                        ability_type,
                        value,
                    );
                }
                true
            }
            QsdReward::AddItem(_reward_target, item, quantity) => quest_reward_add_item(
                quest_system_resources,
                quest_system_parameters,
                quest_parameters,
                item,
                quantity,
            ),
            QsdReward::RemoveItem(_reward_target, item, quantity) => {
                quest_reward_remove_item(quest_system_resources, quest_parameters, item, quantity)
            }
            QsdReward::AddSkill(skill_id) => {
                quest_reward_add_skill(quest_system_resources, quest_parameters, skill_id).is_some()
            }
            QsdReward::RemoveSkill(skill_id) => {
                quest_reward_remove_skill(quest_system_resources, quest_parameters, skill_id)
                    .is_some()
            }
            QsdReward::ResetBasicStats => {
                quest_reward_reset_basic_stats(quest_system_resources, quest_parameters)
            }
            QsdReward::ResetSkills => {
                quest_reward_reset_skills(quest_system_resources, quest_parameters)
            }
            QsdReward::SetQuestSwitch(switch_id, value) => {
                quest_reward_set_quest_switch(quest_parameters, switch_id, value)
            }
            QsdReward::CalculatedExperiencePoints(
                reward_target,
                reward_equation_id,
                base_reward_value,
            ) => quest_reward_calculated_experience_points(
                quest_system_parameters,
                quest_system_resources,
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
                quest_system_parameters,
                quest_system_resources,
                quest_parameters,
                reward_target,
                reward_equation_id,
                base_reward_value,
                item,
                gem,
            ),
            QsdReward::CalculatedMoney(reward_target, reward_equation_id, base_reward_value) => {
                quest_reward_calculated_money(
                    quest_system_resources,
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
                quest_system_parameters,
                quest_parameters,
                ZoneId::new(zone_id as u16).unwrap(),
                Vec3::new(position.x, position.y, 0.0),
            ),
            QsdReward::Trigger(ref name) => {
                quest_parameters.next_trigger_name = Some(name.clone());
                true
            }
            QsdReward::QuestVariable(ref quest_variables) => {
                quest_variables.iter().all(|quest_variable| {
                    quest_reward_quest_variable(
                        quest_system_resources,
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
                quest_system_parameters,
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
                quest_system_parameters,
                quest_system_resources,
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
                quest_reward_set_monster_spawn_state(quest_system_parameters, zone_id, state)
            }
            QsdReward::NpcMessage(message_type, string_id) => quest_reward_npc_message(
                quest_system_parameters,
                quest_system_resources,
                quest_parameters,
                message_type,
                string_id,
            ),
            _ => {
                warn!("Unimplemented quest reward: {:?}", reward);
                false
            } /*
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
            log::trace!(target: "quest", "Reward Failed {:?}", reward);
            return false;
        } else {
            log::trace!(target: "quest", "Reward Success {:?}", reward);
        }
    }

    true
}

pub fn quest_system(
    mut quest_system_parameters: QuestSystemParameters,
    quest_system_resources: QuestSystemResources,
    mut query: Query<QuestSourceEntityQuery>,
    mut quest_trigger_events: EventReader<QuestTriggerEvent>,
) {
    for &QuestTriggerEvent {
        trigger_entity,
        trigger_hash,
    } in quest_trigger_events.iter()
    {
        let mut trigger = quest_system_resources
            .game_data
            .quests
            .get_trigger_by_hash(trigger_hash);
        let mut success = false;

        if let Ok(mut quest_source_entity) = query.get_mut(trigger_entity) {
            let mut quest_parameters = QuestParameters {
                source: &mut quest_source_entity,
                selected_event_object: None,
                selected_npc: None,
                selected_quest_index: None,
                next_trigger_name: None,
            };

            while trigger.is_some() {
                let quest_trigger = trigger.unwrap();

                if quest_trigger_check_conditions(
                    &mut quest_system_parameters,
                    &quest_system_resources,
                    &mut quest_parameters,
                    quest_trigger,
                ) && quest_trigger_apply_rewards(
                    &mut quest_system_parameters,
                    &quest_system_resources,
                    &mut quest_parameters,
                    quest_trigger,
                ) {
                    success = true;

                    if quest_parameters.next_trigger_name.is_some() {
                        trigger = quest_parameters.next_trigger_name.take().and_then(|name| {
                            quest_system_resources
                                .game_data
                                .quests
                                .get_trigger_by_name(&name)
                        });
                    } else {
                        trigger = None;
                    }
                } else {
                    trigger = trigger
                        .unwrap()
                        .next_trigger_name
                        .as_ref()
                        .and_then(|name| {
                            quest_system_resources
                                .game_data
                                .quests
                                .get_trigger_by_name(name)
                        });
                }
            }

            if let Some(game_client) = quest_source_entity.game_client {
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
