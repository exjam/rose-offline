use crate::{
    data::{SkillDatabase, SkillReference},
    game::{
        components::{GameClient, SkillList, SkillPoints, SkillSlot},
        messages::server::{LearnSkillError, LearnSkillSuccess, ServerMessage},
    },
};

fn try_learn_skill(
    skill_database: &SkillDatabase,
    skill: SkillReference,
    skill_list: &mut SkillList,
    skill_points: Option<&mut SkillPoints>,
) -> Result<SkillSlot, LearnSkillError> {
    let skill_data = skill_database
        .get_skill(&skill)
        .ok_or(LearnSkillError::InvalidSkillId)?;

    if skill_list.find_skill(skill_data).is_some() {
        return Err(LearnSkillError::AlreadyLearnt);
    }

    if let Some(skill_points) = skill_points.as_ref() {
        if skill_points.points < skill_data.learn_point_cost {
            return Err(LearnSkillError::SkillPointRequirement);
        }
    }

    // TODO: Check skill job / skill / ability requirements

    let (skill_slot, _) = skill_list
        .add_skill(skill_data)
        .ok_or(LearnSkillError::Full)?;

    if let Some(skill_points) = skill_points {
        skill_points.points -= skill_data.learn_point_cost;
    }

    Ok(skill_slot)
}

pub fn skill_list_try_learn_skill(
    skill_database: &SkillDatabase,
    skill: SkillReference,
    skill_list: &mut SkillList,
    mut skill_points: Option<&mut SkillPoints>,
    game_client: Option<&GameClient>,
) -> Result<SkillSlot, LearnSkillError> {
    let result = try_learn_skill(
        skill_database,
        skill,
        skill_list,
        skill_points.as_deref_mut(),
    );

    if let Some(game_client) = game_client {
        match result {
            Ok(skill_slot) => {
                game_client
                    .server_message_tx
                    .send(ServerMessage::LearnSkillResult(Ok(LearnSkillSuccess {
                        skill_slot,
                        skill,
                        updated_skill_points: skill_points
                            .cloned()
                            .unwrap_or_else(SkillPoints::new),
                    })))
                    .ok();
            }
            Err(error) => {
                game_client
                    .server_message_tx
                    .send(ServerMessage::LearnSkillResult(Err(error)))
                    .ok();
            }
        }
    }

    result
}
