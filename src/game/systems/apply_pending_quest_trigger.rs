use legion::{system, world::SubWorld, Entity, Query};
use log::warn;
use rand::{prelude::ThreadRng, Rng};

use crate::{
    data::{
        formats::qsd::{QsdCondition, QsdReward, QsdRewardCalculatedItem, QsdRewardTarget},
        item::{EquipmentItem, Item},
        ItemReference, QuestTrigger,
    },
    game::{
        components::{BasicStats, CharacterInfo, GameClient, Inventory, Level, Money, QuestState},
        messages::server::{QuestTriggerResult, ServerMessage, UpdateInventory, UpdateMoney},
        resources::{
            PendingQuestTrigger, PendingQuestTriggerList, PendingXp, PendingXpList, WorldRates,
        },
        GameData,
    },
};

struct QuestSourceEntity<'a> {
    entity: &'a Entity,
    game_client: Option<&'a GameClient>,
    inventory: Option<&'a mut Inventory>,
    level: Option<&'a Level>,
    character_info: Option<&'a CharacterInfo>,
    basic_stats: Option<&'a BasicStats>,
    quest_state: Option<&'a mut QuestState>,
}

struct QuestParameters<'a, 'b> {
    source: &'a mut QuestSourceEntity<'b>,
}

struct QuestWorld<'a> {
    game_data: &'a GameData,
    world_rates: &'a WorldRates,
    pending_xp_list: &'a mut PendingXpList,
    rng: ThreadRng,
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

fn quest_trigger_check_conditions(
    _quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match *condition {
            QsdCondition::QuestSwitch(switch_id, value) => {
                quest_condition_quest_switch(quest_parameters, switch_id, value)
            }
            _ => {
                warn!("Unimplemented quest condition: {:?}", condition);
                false
            }
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
            quest_parameters.source.level.map(|x| x.level).unwrap_or(1) as i32,
            quest_parameters
                .source
                .basic_stats
                .map(|x| x.charm)
                .unwrap_or(0) as i32,
            quest_parameters
                .source
                .character_info
                .map(|x| x.fame)
                .unwrap_or(0) as i32,
            quest_world.world_rates.reward_rate,
        );

    quest_world.pending_xp_list.push(PendingXp::new(
        *quest_parameters.source.entity,
        reward_value,
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
    let item = if reward_item.item_type.is_stackable_item() {
        let reward_value = quest_world
            .game_data
            .ability_value_calculator
            .calculate_reward_value(
                reward_equation_id,
                base_reward_value,
                0,
                quest_parameters.source.level.map(|x| x.level).unwrap_or(1) as i32,
                quest_parameters
                    .source
                    .basic_stats
                    .map(|x| x.charm)
                    .unwrap_or(0) as i32,
                quest_parameters
                    .source
                    .character_info
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
                }
            }
        }
    }

    true
}

fn quest_reward_calculated_money(
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
            0, // TODO: This should be the value of the last quest variable of current active quest, and after it resets it to 0
            quest_parameters.source.level.map(|x| x.level).unwrap_or(1) as i32,
            quest_parameters
                .source
                .basic_stats
                .map(|x| x.charm)
                .unwrap_or(0) as i32,
            quest_parameters
                .source
                .character_info
                .map(|x| x.fame)
                .unwrap_or(0) as i32,
            quest_world.world_rates.reward_rate,
        );
    let money = Money(reward_value as i64);

    if let Some(inventory) = quest_parameters.source.inventory.as_mut() {
        if let Ok(_) = inventory.try_add_money(money) {
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

fn quest_trigger_apply_rewards(
    quest_world: &mut QuestWorld,
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for reward in quest_trigger.rewards.iter() {
        let result = match *reward {
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
            _ => {
                warn!("Unimplemented quest reward: {:?}", reward);
                false
            }
        };

        if !result {
            return false;
        }
    }

    true
}

#[allow(clippy::type_complexity)]
#[system]
pub fn apply_pending_quest_trigger(
    world: &mut SubWorld,
    entity_query: &mut Query<(
        Option<&mut Inventory>,
        Option<&Level>,
        Option<&CharacterInfo>,
        Option<&BasicStats>,
        Option<&mut QuestState>,
        Option<&GameClient>,
    )>,
    #[resource] game_data: &GameData,
    #[resource] world_rates: &WorldRates,
    #[resource] pending_quest_trigger_list: &mut PendingQuestTriggerList,
    #[resource] pending_xp_list: &mut PendingXpList,
) {
    let mut quest_world = QuestWorld {
        game_data,
        world_rates,
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

        if let Ok((inventory, level, character_info, basic_stats, quest_state, game_client)) =
            entity_query.get_mut(world, *trigger_entity)
        {
            let mut quest_parameters = QuestParameters {
                source: &mut QuestSourceEntity {
                    entity: trigger_entity,
                    game_client,
                    inventory,
                    level,
                    character_info,
                    basic_stats,
                    quest_state,
                },
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

                trigger = trigger
                    .unwrap()
                    .next_trigger_name
                    .as_ref()
                    .and_then(|name| game_data.quests.get_trigger_by_name(name));
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
