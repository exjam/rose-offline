use legion::{system, world::SubWorld, Entity, Query};
use log::warn;

use crate::{
    data::{
        formats::qsd::{QsdCondition, QsdReward},
        QuestTrigger,
    },
    game::{
        components::{GameClient, QuestState},
        messages::server::{QuestTriggerResult, ServerMessage},
        resources::{PendingQuestTrigger, PendingQuestTriggerList},
        GameData,
    },
};

struct QuestSourceEntity<'a> {
    entity: &'a Entity,
    quest_state: Option<&'a mut QuestState>,
}

struct QuestParameters<'a, 'b> {
    source: &'a mut QuestSourceEntity<'b>,
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
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match condition {
            &QsdCondition::QuestSwitch(switch_id, value) => {
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

fn quest_trigger_apply_rewards(
    quest_parameters: &mut QuestParameters,
    quest_trigger: &QuestTrigger,
) -> bool {
    for reward in quest_trigger.rewards.iter() {
        let result = match reward {
            &QsdReward::SetQuestSwitch(switch_id, value) => {
                quest_reward_set_quest_switch(quest_parameters, switch_id, value)
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

#[system]
pub fn apply_pending_quest_trigger(
    world: &mut SubWorld,
    entity_query: &mut Query<(Option<&mut QuestState>, Option<&GameClient>)>,
    #[resource] game_data: &GameData,
    #[resource] pending_quest_trigger_list: &mut PendingQuestTriggerList,
) {
    for PendingQuestTrigger {
        trigger_entity,
        trigger_hash,
    } in pending_quest_trigger_list.iter()
    {
        let mut trigger = game_data.quests.get_trigger_by_hash(*trigger_hash);
        let mut success = false;

        if let Ok((quest_state, game_client)) = entity_query.get_mut(world, *trigger_entity) {
            let mut quest_parameters = QuestParameters {
                source: &mut QuestSourceEntity {
                    entity: trigger_entity,
                    quest_state,
                },
            };

            while trigger.is_some() {
                let quest_trigger = trigger.unwrap();

                if quest_trigger_check_conditions(&mut quest_parameters, quest_trigger)
                    && quest_trigger_apply_rewards(&mut quest_parameters, quest_trigger)
                {
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
