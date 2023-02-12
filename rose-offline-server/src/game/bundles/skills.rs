use bevy::ecs::prelude::Mut;

use rose_data::{SkillData, SkillDatabase, SkillId};

use crate::game::{
    components::{GameClient, SkillList, SkillPoints, SkillSlot},
    messages::server::{
        LearnSkillError, LearnSkillSuccess, LevelUpSkillError, LevelUpSkillResult, ServerMessage,
    },
};

fn check_skill_point_requirements(
    skill_data: &SkillData,
    skill_points: Option<&SkillPoints>,
) -> bool {
    skill_points.map_or(true, |skill_points| {
        skill_points.points >= skill_data.learn_point_cost
    })
}

fn try_learn_skill(
    skill_database: &SkillDatabase,
    skill_id: SkillId,
    skill_list: &mut SkillList,
    skill_points: Option<&mut Mut<SkillPoints>>,
) -> Result<SkillSlot, LearnSkillError> {
    let skill_data = skill_database
        .get_skill(skill_id)
        .ok_or(LearnSkillError::InvalidSkillId)?;

    if skill_list.find_skill_exact(skill_data).is_some() {
        return Err(LearnSkillError::AlreadyLearnt);
    }

    if !check_skill_point_requirements(skill_data, skill_points.as_deref().map(|x| &**x)) {
        return Err(LearnSkillError::SkillPointRequirement);
    }

    // TODO: Check job requirement
    // TODO: Check skill requirement
    // TODO: Check ability requirement

    let (skill_slot, _) = skill_list
        .add_skill(skill_data)
        .ok_or(LearnSkillError::Full)?;

    if let Some(skill_points) = skill_points {
        skill_points.points -= skill_data.learn_point_cost;
    }

    Ok(skill_slot)
}

fn try_level_up_skill(
    skill_database: &SkillDatabase,
    skill_slot: SkillSlot,
    skill_list: &mut SkillList,
    mut skill_points: Option<&mut Mut<SkillPoints>>,
) -> Result<SkillId, LevelUpSkillError> {
    let current_skill_id = skill_list
        .get_skill(skill_slot)
        .ok_or(LevelUpSkillError::Failed)?;
    let next_skill_id = SkillId::new(current_skill_id.get() + 1).unwrap();

    let current_skill_data = skill_database
        .get_skill(current_skill_id)
        .ok_or(LevelUpSkillError::Failed)?;
    let next_skill_data = skill_database
        .get_skill(next_skill_id)
        .ok_or(LevelUpSkillError::Failed)?;

    if next_skill_data.base_skill_id != current_skill_data.base_skill_id {
        return Err(LevelUpSkillError::Failed);
    }

    if next_skill_data.level != current_skill_data.level + 1 {
        return Err(LevelUpSkillError::Failed);
    }

    if !check_skill_point_requirements(next_skill_data, skill_points.as_deref().map(|x| &**x)) {
        return Err(LevelUpSkillError::SkillPointRequirement);
    }

    // TODO: Check job requirement
    // TODO: Check skill requirement
    // TODO: Check ability requirement

    let skill_list_slot = skill_list
        .get_slot_mut(skill_slot)
        .ok_or(LevelUpSkillError::Failed)?;
    *skill_list_slot = Some(next_skill_id);

    if let Some(skill_points) = skill_points.as_deref_mut() {
        skill_points.points -= next_skill_data.learn_point_cost;
    }

    Ok(next_skill_id)
}

pub fn skill_list_try_learn_skill(
    skill_database: &SkillDatabase,
    skill_id: SkillId,
    skill_list: &mut Mut<SkillList>,
    mut skill_points: Option<&mut Mut<SkillPoints>>,
    game_client: Option<&GameClient>,
) -> Result<SkillSlot, LearnSkillError> {
    let result = try_learn_skill(
        skill_database,
        skill_id,
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
                        skill_id: Some(skill_id),
                        updated_skill_points: skill_points
                            .as_deref()
                            .map_or_else(SkillPoints::default, |skill_points| **skill_points),
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

pub fn skill_list_try_level_up_skill(
    skill_database: &SkillDatabase,
    skill_slot: SkillSlot,
    skill_list: &mut SkillList,
    mut skill_points: Option<&mut Mut<SkillPoints>>,
    game_client: Option<&GameClient>,
) -> Result<SkillId, LevelUpSkillError> {
    let result = try_level_up_skill(
        skill_database,
        skill_slot,
        skill_list,
        skill_points.as_deref_mut(),
    );

    if let Some(game_client) = game_client {
        let updated_skill_points = skill_points
            .as_deref()
            .map_or_else(SkillPoints::default, |skill_points| **skill_points);

        match result {
            Ok(skill_id) => {
                game_client
                    .server_message_tx
                    .send(ServerMessage::LevelUpSkillResult(LevelUpSkillResult {
                        result: Ok((skill_slot, skill_id)),
                        updated_skill_points,
                    }))
                    .ok();
            }
            Err(error) => {
                game_client
                    .server_message_tx
                    .send(ServerMessage::LevelUpSkillResult(LevelUpSkillResult {
                        result: Err(error),
                        updated_skill_points,
                    }))
                    .ok();
            }
        }
    }

    result
}
