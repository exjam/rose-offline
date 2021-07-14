use legion::{system, world::SubWorld, Entity, Query};

use crate::{
    data::{
        formats::qsd::{QsdCondition, QsdReward},
        QuestTrigger,
    },
    game::{
        components::{ClientEntity, GameClient},
        messages::server::{QuestTriggerResult, ServerMessage},
        resources::{PendingQuestTrigger, PendingQuestTriggerList},
        GameData,
    },
};

struct QsdParameters {}

fn quest_trigger_check_conditions(quest_trigger: &QuestTrigger) -> bool {
    for condition in quest_trigger.conditions.iter() {
        let result = match condition {
            QsdCondition::SelectQuest(_) => false,
            QsdCondition::QuestVariable(_) => false,
            QsdCondition::AbilityValue(_) => false,
            QsdCondition::QuestItems(_) => false,
            QsdCondition::Party(_) => false,
            QsdCondition::Position(_, _, _) => false,
            QsdCondition::WorldTime(_) => false,
            QsdCondition::QuestTimeRemaining(_, _) => false,
            QsdCondition::HasSkill(_, _) => false,
            QsdCondition::RandomPercent(_) => false,
            QsdCondition::ObjectVariable(_) => false,
            QsdCondition::SelectEventObject(_) => false,
            QsdCondition::SelectNpc(_) => false,
            QsdCondition::QuestSwitch(_, _) => false,
            QsdCondition::PartyMemberCount(_) => false,
            QsdCondition::ObjectZoneTime(_, _) => false,
            QsdCondition::CompareNpcVariables(_, _, _) => false,
            QsdCondition::MonthDayTime(_) => false,
            QsdCondition::WeekDayTime(_) => false,
            QsdCondition::TeamNumber(_) => false,
            QsdCondition::ObjectDistance(_, _) => false,
            QsdCondition::ServerChannelNumber(_) => false,
            QsdCondition::InClan(_) => false,
            QsdCondition::ClanPosition(_, _) => false,
            QsdCondition::ClanPointContribution(_, _) => false,
            QsdCondition::ClanLevel(_, _) => false,
            QsdCondition::ClanPoints(_, _) => false,
            QsdCondition::ClanMoney(_, _) => false,
            QsdCondition::ClanMemberCount(_, _) => false,
            QsdCondition::HasClanSkill(_, _) => false,
        };

        if !result {
            return false;
        }
    }

    true
}

fn quest_trigger_apply_rewards(quest_trigger: &QuestTrigger) -> bool {
    for reward in quest_trigger.rewards.iter() {
        let result = match reward {
            QsdReward::Quest(_, _) => false,
            QsdReward::AddItem(_, _, _) => false,
            QsdReward::RemoveItem(_, _, _) => false,
            QsdReward::QuestVariable(_) => false,
            QsdReward::AbilityValue(_) => false,
            QsdReward::CalculatedExperiencePoints(_, _, _) => false,
            QsdReward::CalculatedMoney(_, _, _) => false,
            QsdReward::CalculatedItem(_, _) => false,
            QsdReward::SetHealthManaPercent(_, _, _) => false,
            QsdReward::Teleport(_, _, _) => false,
            QsdReward::SpawnNpc(_) => false,
            QsdReward::Trigger(_) => false,
            QsdReward::ResetBasicStats => false,
            QsdReward::ObjectVariable(_) => false,
            QsdReward::NpcMessage(_, _) => false,
            QsdReward::TriggerAfterDelayForObject(_, _, _) => false,
            QsdReward::AddSkill(_) => false,
            QsdReward::RemoveSkill(_) => false,
            QsdReward::SetQuestSwitch(_, _) => false,
            QsdReward::ClearSwitchGroup(_) => false,
            QsdReward::ClearAllSwitches => false,
            QsdReward::FormatAnnounceMessage(_, _) => false,
            QsdReward::TriggerForZoneTeam(_, _, _) => false,
            QsdReward::SetTeamNumber(_) => false,
            QsdReward::SetRevivePosition(_) => false,
            QsdReward::SetMonsterSpawnState(_, _) => false,
            QsdReward::ClanLevel(_, _) => false,
            QsdReward::ClanMoney(_, _) => false,
            QsdReward::ClanPoints(_, _) => false,
            QsdReward::AddClanSkill(_) => false,
            QsdReward::RemoveClanSkill(_) => false,
            QsdReward::ClanPointContribution(_, _) => false,
            QsdReward::TeleportNearbyClanMembers(_, _, _) => false,
            QsdReward::CallLuaFunction(_) => false,
            QsdReward::ResetSkills => false,
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
    entity_query: &mut Query<(&ClientEntity, Option<&GameClient>)>,
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

        if let Ok((client_entity, game_client)) = entity_query.get(world, *trigger_entity) {
            while trigger.is_some() {
                let quest_trigger = trigger.unwrap();

                if quest_trigger_check_conditions(quest_trigger)
                    && quest_trigger_apply_rewards(quest_trigger)
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
