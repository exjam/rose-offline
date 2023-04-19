use rose_data::{JobId, SkillData, SkillId};
use rose_game_common::components::{
    AbilityValues, CharacterInfo, ExperiencePoints, HealthPoints, Inventory, Level, ManaPoints,
    MoveSpeed, Stamina, StatPoints, Team, UnionMembership,
};

use crate::game::{
    bundles::ability_values_get_value,
    components::{GameClient, SkillList, SkillPoints, SkillSlot},
    messages::server::{
        LearnSkillError, LearnSkillSuccess, LevelUpSkillError, LevelUpSkillResult, ServerMessage,
    },
    GameData,
};

pub struct SkillListBundle<'w> {
    pub skill_list: &'w mut SkillList,
    pub skill_points: Option<&'w mut SkillPoints>,
    pub game_client: Option<&'w GameClient>,

    pub ability_values: &'w AbilityValues,
    pub level: &'w Level,
    pub move_speed: Option<&'w MoveSpeed>,
    pub team: Option<&'w Team>,
    pub character_info: Option<&'w CharacterInfo>,
    pub experience_points: Option<&'w ExperiencePoints>,
    pub inventory: Option<&'w Inventory>,
    pub stamina: Option<&'w Stamina>,
    pub stat_points: Option<&'w StatPoints>,
    pub union_membership: Option<&'w UnionMembership>,
    pub health_points: Option<&'w HealthPoints>,
    pub mana_points: Option<&'w ManaPoints>,
}

fn check_skill_point_requirements(
    skill_data: &SkillData,
    skill_points: Option<&SkillPoints>,
) -> bool {
    skill_points.map_or(true, |skill_points| {
        skill_points.points >= skill_data.learn_point_cost
    })
}

fn check_skill_job_requirements(
    game_data: &GameData,
    skill_data: &SkillData,
    character_info: &CharacterInfo,
) -> bool {
    let Some(job_class_id) = skill_data.required_job_class else {
        return true;
    };
    let Some(job_class) = game_data.job_class.get(job_class_id) else {
        return true;
    };

    job_class.jobs.contains(&JobId::new(character_info.job))
}

fn check_skill_skill_requirements(
    game_data: &GameData,
    skill_data: &SkillData,
    skill_list: &SkillList,
) -> bool {
    for &(required_skill_id, required_level) in skill_data.required_skills.iter() {
        if let Some(required_skill_data) = game_data.skills.get_skill(
            SkillId::new(required_skill_id.get() + required_level.max(1) as u16 - 1).unwrap(),
        ) {
            let Some((_, _, skill_level)) = skill_list.find_skill_level(
                &game_data.skills,
                required_skill_data
                    .base_skill_id
                    .unwrap_or(required_skill_id),
            ) else {
                return false;
            };

            if skill_level < required_level as u32 {
                return false;
            }
        }
    }

    true
}

fn check_skill_ability_requirements(skill_data: &SkillData, skill_user: &SkillListBundle) -> bool {
    for &(ability_type, value) in skill_data.required_ability.iter() {
        let Some(current_value) =
        ability_values_get_value(
            ability_type,
            Some(skill_user.ability_values),
            Some(skill_user.level),
            skill_user.move_speed,
            skill_user.team,
            skill_user.character_info,
            skill_user.experience_points,
            skill_user.inventory,
            skill_user.skill_points.as_deref(),
            skill_user.stamina,
            skill_user.stat_points,
            skill_user.union_membership,
            skill_user.health_points,
            skill_user.mana_points,
        ) else {
            return false;
        };

        if current_value < value {
            return false;
        }
    }

    true
}

pub fn can_learn_skill(
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_id: SkillId,
) -> Result<SkillPoints, LearnSkillError> {
    let skill_data = game_data
        .skills
        .get_skill(skill_id)
        .ok_or(LearnSkillError::InvalidSkillId)?;

    if skill_user.skill_list.find_skill_exact(skill_data).is_some() {
        return Err(LearnSkillError::AlreadyLearnt);
    }

    if !check_skill_point_requirements(skill_data, skill_user.skill_points.as_deref()) {
        return Err(LearnSkillError::SkillPointRequirement);
    }

    if let Some(character_info) = skill_user.character_info {
        if !check_skill_job_requirements(game_data, skill_data, character_info) {
            return Err(LearnSkillError::JobRequirement);
        }
    }

    if !check_skill_skill_requirements(game_data, skill_data, skill_user.skill_list) {
        return Err(LearnSkillError::SkillRequirement);
    }

    if !check_skill_ability_requirements(skill_data, skill_user) {
        return Err(LearnSkillError::AbilityRequirement);
    }

    Ok(SkillPoints::new(skill_data.learn_point_cost))
}

fn try_learn_skill(
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_id: SkillId,
) -> Result<SkillSlot, LearnSkillError> {
    let skill_data = game_data
        .skills
        .get_skill(skill_id)
        .ok_or(LearnSkillError::InvalidSkillId)?;

    let skill_point_cost = can_learn_skill(game_data, skill_user, skill_id)?;

    let (skill_slot, _) = skill_user
        .skill_list
        .add_skill(skill_data)
        .ok_or(LearnSkillError::Full)?;

    if let Some(skill_points) = skill_user.skill_points.as_mut() {
        skill_points.points -= skill_point_cost.points;
    }

    Ok(skill_slot)
}

pub fn can_level_up_skill(
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_slot: SkillSlot,
) -> Result<SkillPoints, LevelUpSkillError> {
    let current_skill_id = skill_user
        .skill_list
        .get_skill(skill_slot)
        .ok_or(LevelUpSkillError::Failed)?;
    let next_skill_id = SkillId::new(current_skill_id.get() + 1).unwrap();

    let current_skill_data = game_data
        .skills
        .get_skill(current_skill_id)
        .ok_or(LevelUpSkillError::Failed)?;
    let next_skill_data = game_data
        .skills
        .get_skill(next_skill_id)
        .ok_or(LevelUpSkillError::Failed)?;

    if next_skill_data.base_skill_id != current_skill_data.base_skill_id {
        return Err(LevelUpSkillError::Failed);
    }

    if next_skill_data.level != current_skill_data.level + 1 {
        return Err(LevelUpSkillError::Failed);
    }

    if !check_skill_point_requirements(next_skill_data, skill_user.skill_points.as_deref()) {
        return Err(LevelUpSkillError::SkillPointRequirement);
    }

    if let Some(character_info) = skill_user.character_info {
        if !check_skill_job_requirements(game_data, next_skill_data, character_info) {
            return Err(LevelUpSkillError::JobRequirement);
        }
    }

    if !check_skill_skill_requirements(game_data, next_skill_data, skill_user.skill_list) {
        return Err(LevelUpSkillError::SkillRequirement);
    }

    if !check_skill_ability_requirements(next_skill_data, skill_user) {
        return Err(LevelUpSkillError::AbilityRequirement);
    }

    Ok(SkillPoints::new(next_skill_data.learn_point_cost))
}

fn try_level_up_skill(
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_slot: SkillSlot,
) -> Result<SkillId, LevelUpSkillError> {
    let current_skill_id = skill_user
        .skill_list
        .get_skill(skill_slot)
        .ok_or(LevelUpSkillError::Failed)?;
    let next_skill_id = SkillId::new(current_skill_id.get() + 1).unwrap();

    let skill_point_cost = can_level_up_skill(game_data, skill_user, skill_slot)?;

    let skill_list_slot = skill_user
        .skill_list
        .get_slot_mut(skill_slot)
        .ok_or(LevelUpSkillError::Failed)?;
    *skill_list_slot = Some(next_skill_id);

    if let Some(skill_points) = skill_user.skill_points.as_deref_mut() {
        skill_points.points -= skill_point_cost.points;
    }

    Ok(next_skill_id)
}

pub fn skill_list_try_learn_skill(
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_id: SkillId,
) -> Result<SkillSlot, LearnSkillError> {
    let result = try_learn_skill(game_data, skill_user, skill_id);

    if let Some(game_client) = skill_user.game_client {
        match result {
            Ok(skill_slot) => {
                game_client
                    .server_message_tx
                    .send(ServerMessage::LearnSkillResult(Ok(LearnSkillSuccess {
                        skill_slot,
                        skill_id: Some(skill_id),
                        updated_skill_points: skill_user
                            .skill_points
                            .as_deref()
                            .map_or_else(SkillPoints::default, |skill_points| *skill_points),
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
    game_data: &GameData,
    skill_user: &mut SkillListBundle,
    skill_slot: SkillSlot,
) -> Result<SkillId, LevelUpSkillError> {
    let result = try_level_up_skill(game_data, skill_user, skill_slot);

    if let Some(game_client) = skill_user.game_client {
        let updated_skill_points = skill_user
            .skill_points
            .as_deref()
            .map_or_else(SkillPoints::default, |skill_points| *skill_points);

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
