use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query};
use log::warn;
use nalgebra::Point3;
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::{
        ability::AbilityType,
        formats::qsd::{
            QsdCondition, QsdConditionOperator, QsdConditionQuestItem, QsdReward,
            QsdRewardCalculatedItem, QsdRewardOperator, QsdRewardQuestAction, QsdRewardTarget,
        },
        item::{EquipmentItem, Item},
        ItemReference, QuestTrigger, SkillReference, WorldTicks, ZoneReference,
    },
    game::{
        bundles::{
            ability_values_add_value, ability_values_get_value, ability_values_set_value,
            client_entity_teleport_zone, skill_list_try_learn_skill,
        },
        components::{
            AbilityValues, ActiveQuest, BasicStats, CharacterInfo, ClientEntity, Equipment,
            EquipmentIndex, ExperiencePoints, GameClient, Inventory, Level, Money, MoveSpeed,
            Position, QuestState, SkillList, SkillPoints, StatPoints, Team, UnionMembership,
        },
        messages::server::{QuestTriggerResult, ServerMessage, UpdateInventory, UpdateMoney},
        resources::{
            ClientEntityList, PendingQuestTrigger, PendingQuestTriggerList, PendingXp,
            PendingXpList, WorldRates, WorldTime,
        },
        GameData,
    },
};

struct QuestSourceEntity<'a> {
    entity: &'a Entity,
    game_client: Option<&'a GameClient>,
    client_entity: Option<&'a ClientEntity>,
    ability_values: Option<&'a mut AbilityValues>,
    basic_stats: Option<&'a mut BasicStats>,
    character_info: Option<&'a mut CharacterInfo>,
    equipment: Option<&'a Equipment>,
    experience_points: Option<&'a mut ExperiencePoints>,
    inventory: Option<&'a mut Inventory>,
    level: Option<&'a mut Level>,
    move_speed: Option<&'a mut MoveSpeed>,
    position: Option<&'a Position>,
    quest_state: Option<&'a mut QuestState>,
    skill_list: Option<&'a mut SkillList>,
    skill_points: Option<&'a mut SkillPoints>,
    stat_points: Option<&'a mut StatPoints>,
    team: Option<&'a mut Team>,
    union_membership: Option<&'a mut UnionMembership>,
}

struct QuestParameters<'a, 'b> {
    source: &'a mut QuestSourceEntity<'b>,
    selected_quest_index: Option<usize>,
    next_trigger_name: Option<String>,
}

struct QuestWorld<'a> {
    cmd: &'a mut CommandBuffer,
    client_entity_list: &'a mut ClientEntityList,
    game_data: &'a GameData,
    world_rates: &'a WorldRates,
    world_time: &'a WorldTime,
    pending_xp_list: &'a mut PendingXpList,
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
        .map(|time_limit| quest_world.world_time.now + time_limit)
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
    quest_parameters: &mut QuestParameters,
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
    quest_parameters: &mut QuestParameters,
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
    quest_parameters: &mut QuestParameters,
    ability_values: &[(AbilityType, QsdConditionOperator, i32)],
) -> bool {
    for &(ability_type, operator, compare_value) in ability_values {
        let current_value = ability_values_get_value(
            ability_type,
            quest_parameters.source.ability_values.as_deref(),
            quest_parameters.source.character_info.as_deref(),
            quest_parameters.source.experience_points.as_deref(),
            quest_parameters.source.inventory.as_deref(),
            quest_parameters.source.level.as_deref(),
            quest_parameters.source.move_speed.as_deref(),
            quest_parameters.source.stat_points.as_deref(),
            quest_parameters.source.skill_points.as_deref(),
            quest_parameters.source.team.as_deref(),
            quest_parameters.source.union_membership.as_deref(),
        )
        .unwrap_or(0);

        if !quest_condition_operator(operator, current_value, compare_value) {
            return false;
        }
    }

    true
}

fn quest_trigger_check_conditions(
    _quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match condition {
            QsdCondition::AbilityValue(ability_values) => {
                quest_condition_ability_values(quest_parameters, ability_values)
            }
            &QsdCondition::SelectQuest(quest_id) => {
                quest_condition_select_quest(quest_parameters, quest_id)
            }
            QsdCondition::QuestItems(items) => quest_condition_quest_items(quest_parameters, items),
            &QsdCondition::QuestSwitch(switch_id, value) => {
                quest_condition_quest_switch(quest_parameters, switch_id, value)
            }
            _ => {
                warn!("Unimplemented quest condition: {:?}", condition);
                false
            } /*
              QsdCondition::QuestVariable(_) => todo!(),
              QsdCondition::Party(_) => todo!(),
              QsdCondition::Position(_, _, _) => todo!(),
              QsdCondition::WorldTime(_) => todo!(),
              QsdCondition::QuestTimeRemaining(_, _) => todo!(),
              QsdCondition::HasSkill(_, _) => todo!(),
              QsdCondition::RandomPercent(_) => todo!(),
              QsdCondition::ObjectVariable(_) => todo!(),
              QsdCondition::SelectEventObject(_) => todo!(),
              QsdCondition::SelectNpc(_) => todo!(),
              QsdCondition::PartyMemberCount(_) => todo!(),
              QsdCondition::ObjectZoneTime(_, _) => todo!(),
              QsdCondition::CompareNpcVariables(_, _, _) => todo!(),
              QsdCondition::MonthDayTime(_) => todo!(),
              QsdCondition::WeekDayTime(_) => todo!(),
              QsdCondition::TeamNumber(_) => todo!(),
              QsdCondition::ObjectDistance(_, _) => todo!(),
              QsdCondition::ServerChannelNumber(_) => todo!(),
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
            quest_parameters
                .source
                .level
                .as_ref()
                .map(|x| x.level)
                .unwrap_or(1) as i32,
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

    quest_world.pending_xp_list.push(PendingXp::new(
        *quest_parameters.source.entity,
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
                quest_parameters
                    .source
                    .level
                    .as_ref()
                    .map(|x| x.level)
                    .unwrap_or(1) as i32,
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

fn get_quest_calculated_money_dup_count_var<'a>(
    selected_quest_index: Option<usize>,
    quest_state: &'a mut Option<&mut QuestState>,
) -> Option<&'a mut u16> {
    let quest_index = selected_quest_index?;
    let quest_state = quest_state.as_mut()?;
    let active_quest = quest_state.get_quest_mut(quest_index)?;
    active_quest.variables.last_mut()
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
        &mut quest_parameters.source.quest_state,
    );

    let reward_value = quest_world
        .game_data
        .ability_value_calculator
        .calculate_reward_value(
            reward_equation_id,
            base_reward_value,
            dup_count_var.as_ref().map_or(0, |x| **x) as i32,
            quest_parameters
                .source
                .level
                .as_ref()
                .map(|x| x.level)
                .unwrap_or(1) as i32,
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
            if let Some(dup_count_var) = dup_count_var {
                *dup_count_var = 0u16;
            }

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

fn quest_reward_add_skill(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    skill_id: usize,
) -> Option<()> {
    let skill = SkillReference(skill_id);
    if let Some(skill_list) = quest_parameters.source.skill_list.as_deref_mut() {
        skill_list_try_learn_skill(
            quest_world.game_data.skills.as_ref(),
            skill,
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

fn quest_reward_teleport(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    new_zone: ZoneReference,
    new_position: Point3<f32>,
) -> bool {
    if let (Some(client_entity), Some(position)) = (
        quest_parameters.source.client_entity,
        quest_parameters.source.position,
    ) {
        client_entity_teleport_zone(
            quest_world.cmd,
            quest_world.client_entity_list,
            quest_parameters.source.entity,
            client_entity,
            position,
            Position::new(new_position, new_zone.0 as u16),
            quest_parameters.source.game_client,
        );
        true
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
        let result = match reward {
            QsdReward::Quest(action) => {
                quest_reward_quest_action(quest_world, quest_parameters, action)
            }
            QsdReward::AbilityValue(values) => {
                for (ability_type, reward_operator, value) in values {
                    match reward_operator {
                        QsdRewardOperator::Set => ability_values_set_value(
                            *ability_type,
                            *value,
                            quest_parameters.source.basic_stats.as_deref_mut(),
                            quest_parameters.source.character_info.as_deref_mut(),
                            quest_parameters.source.union_membership.as_deref_mut(),
                            quest_parameters.source.game_client.as_deref(),
                        ),
                        QsdRewardOperator::Add => ability_values_add_value(
                            *ability_type,
                            *value,
                            quest_parameters.source.basic_stats.as_deref_mut(),
                            quest_parameters.source.inventory.as_deref_mut(),
                            quest_parameters.source.stat_points.as_deref_mut(),
                            quest_parameters.source.skill_points.as_deref_mut(),
                            quest_parameters.source.union_membership.as_deref_mut(),
                            quest_parameters.source.game_client.as_deref(),
                        ),
                        QsdRewardOperator::Subtract => ability_values_add_value(
                            *ability_type,
                            -*value,
                            quest_parameters.source.basic_stats.as_deref_mut(),
                            quest_parameters.source.inventory.as_deref_mut(),
                            quest_parameters.source.stat_points.as_deref_mut(),
                            quest_parameters.source.skill_points.as_deref_mut(),
                            quest_parameters.source.union_membership.as_deref_mut(),
                            quest_parameters.source.game_client.as_deref(),
                        ),
                        QsdRewardOperator::Zero => ability_values_set_value(
                            *ability_type,
                            0,
                            quest_parameters.source.basic_stats.as_deref_mut(),
                            quest_parameters.source.character_info.as_deref_mut(),
                            quest_parameters.source.union_membership.as_deref_mut(),
                            quest_parameters.source.game_client.as_deref(),
                        ),
                        QsdRewardOperator::One => ability_values_set_value(
                            *ability_type,
                            1,
                            quest_parameters.source.basic_stats.as_deref_mut(),
                            quest_parameters.source.character_info.as_deref_mut(),
                            quest_parameters.source.union_membership.as_deref_mut(),
                            quest_parameters.source.game_client.as_deref(),
                        ),
                    };
                }
                true
            }
            &QsdReward::AddItem(_reward_target, item, quantity) => {
                quest_reward_add_item(quest_parameters, item, quantity)
            }
            &QsdReward::AddSkill(skill_id) => {
                quest_reward_add_skill(quest_world, quest_parameters, skill_id).is_some()
            }
            &QsdReward::SetQuestSwitch(switch_id, value) => {
                quest_reward_set_quest_switch(quest_parameters, switch_id, value)
            }
            &QsdReward::CalculatedExperiencePoints(
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
            &QsdReward::CalculatedItem(
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
            &QsdReward::CalculatedMoney(reward_target, reward_equation_id, base_reward_value) => {
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
            &QsdReward::Teleport(_reward_target, zone_id, position) => quest_reward_teleport(
                quest_world,
                quest_parameters,
                ZoneReference(zone_id),
                Point3::new(position.x, position.y, 0.0),
            ),
            QsdReward::Trigger(name) => {
                quest_parameters.next_trigger_name = Some(name.clone());
                true
            }
            _ => {
                warn!("Unimplemented quest reward: {:?}", reward);
                false
            } /*
              QsdReward::RemoveItem(_, _, _) => todo!(),
              QsdReward::QuestVariable(_) => todo!(),
              QsdReward::SetHealthManaPercent(_, _, _) => todo!(),
              QsdReward::SpawnNpc(_) => todo!(),
              QsdReward::ResetBasicStats => todo!(),
              QsdReward::ObjectVariable(_) => todo!(),
              QsdReward::NpcMessage(_, _) => todo!(),
              QsdReward::TriggerAfterDelayForObject(_, _, _) => todo!(),
              QsdReward::RemoveSkill(_) => todo!(),
              QsdReward::ClearSwitchGroup(_) => todo!(),
              QsdReward::ClearAllSwitches => todo!(),
              QsdReward::FormatAnnounceMessage(_, _) => todo!(),
              QsdReward::TriggerForZoneTeam(_, _, _) => todo!(),
              QsdReward::SetTeamNumber(_) => todo!(),
              QsdReward::SetRevivePosition(_) => todo!(),
              QsdReward::SetMonsterSpawnState(_, _) => todo!(),
              QsdReward::ClanLevel(_, _) => todo!(),
              QsdReward::ClanMoney(_, _) => todo!(),
              QsdReward::ClanPoints(_, _) => todo!(),
              QsdReward::AddClanSkill(_) => todo!(),
              QsdReward::RemoveClanSkill(_) => todo!(),
              QsdReward::ClanPointContribution(_, _) => todo!(),
              QsdReward::TeleportNearbyClanMembers(_, _, _) => todo!(),
              QsdReward::ResetSkills => todo!(),
              */
        };

        if !result {
            return false;
        }
    }

    true
}

#[allow(clippy::type_complexity)]
#[system]
pub fn quest(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    entity_query: &mut Query<(
        Option<&GameClient>,
        Option<&ClientEntity>,
        Option<&mut AbilityValues>,
        Option<&mut BasicStats>,
        Option<&mut CharacterInfo>,
        Option<&Equipment>,
        Option<&mut ExperiencePoints>,
        Option<&mut Inventory>,
        Option<&mut Level>,
        Option<&mut MoveSpeed>,
        Option<&Position>,
        Option<&mut QuestState>,
        Option<&mut SkillList>,
        Option<&mut SkillPoints>,
        Option<&mut StatPoints>,
        Option<&mut Team>,
        Option<&mut UnionMembership>,
    )>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
    #[resource] world_rates: &WorldRates,
    #[resource] pending_quest_trigger_list: &mut PendingQuestTriggerList,
    #[resource] pending_xp_list: &mut PendingXpList,
    #[resource] world_time: &WorldTime,
) {
    let mut quest_world = QuestWorld {
        cmd,
        client_entity_list,
        game_data,
        world_rates,
        world_time,
        pending_xp_list,
        rng: rand::thread_rng(),
    };

    for PendingQuestTrigger {
        trigger_entity,
        trigger_hash,
    } in pending_quest_trigger_list.iter()
    {
        let mut trigger = game_data.quests.get_trigger_by_hash(*trigger_hash);
        let mut success = false;

        if let Ok((
            game_client,
            client_entity,
            ability_values,
            basic_stats,
            character_info,
            equipment,
            experience_points,
            inventory,
            level,
            move_speed,
            position,
            quest_state,
            skill_list,
            skill_points,
            stat_points,
            team,
            union_membership,
        )) = entity_query.get_mut(world, *trigger_entity)
        {
            let mut quest_parameters = QuestParameters {
                source: &mut QuestSourceEntity {
                    entity: trigger_entity,
                    game_client,
                    client_entity,
                    ability_values,
                    basic_stats,
                    character_info,
                    equipment,
                    experience_points,
                    inventory,
                    level,
                    move_speed,
                    position,
                    quest_state,
                    skill_list,
                    skill_points,
                    stat_points,
                    team,
                    union_membership,
                },
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
                        trigger_hash: *trigger_hash,
                    }))
                    .ok();
            }
        }
    }

    pending_quest_trigger_list.clear();
}
