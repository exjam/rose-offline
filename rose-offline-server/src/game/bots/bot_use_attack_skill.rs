use std::time::Duration;

use bevy::{
    prelude::{Commands, Component, Query, Res, With},
    time::Time,
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};
use rand::Rng;

use crate::game::{
    bundles::{skill_can_target_entity, skill_can_use, SkillCasterBundle, SkillTargetBundle},
    components::{Command, CommandData, NextCommand, SkillList},
    GameData,
};

use super::{BotCombatTarget, BotQueryFilterAlive};

const DEAD_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ShouldUseAttackSkill {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct UseAttackSkill;

pub fn score_should_use_attack_skill(
    mut query: Query<(&ShouldUseAttackSkill, &Actor, &mut Score)>,
    query_entity: Query<
        (
            &BotCombatTarget,
            &SkillList,
            SkillCasterBundle,
            Option<&UseAttackSkill>,
        ),
        BotQueryFilterAlive,
    >,
    query_target: Query<SkillTargetBundle>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Some(now) = time.last_update() else {
        return;
    };
    let mut rng = rand::thread_rng();

    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((bot_combat_target, skill_list, skill_caster, is_using_skill)) = query_entity.get(entity) else {
            continue;
        };

        if is_using_skill.is_some() {
            score.set(scorer.score);
            continue;
        }

        let Ok(skill_target) =query_target.get(bot_combat_target.entity) else {
            continue;
        };

        let Some(mana_points) = skill_caster.mana_points else {
            continue;
        };

        if (mana_points.mp as f32 / skill_caster.ability_values.get_max_mana() as f32) < 0.5 {
            continue;
        }

        if skill_target.health_points.hp < 250 {
            continue;
        }

        let Some(active_skill_page) = skill_list.pages.get(1) else {
            continue;
        };

        if rng.gen_range(0..=100) < 95 {
            continue;
        }

        for skill_id in active_skill_page.skills.iter().filter_map(|x| x.as_ref()) {
            if let Some(skill_data) = game_data.skills.get_skill(*skill_id) {
                if skill_can_use(now, &game_data, &skill_caster, skill_data)
                    && skill_can_target_entity(&skill_caster, &skill_target, skill_data)
                {
                    score.set(scorer.score);
                    break;
                }
            }
        }
    }
}

pub fn action_use_attack_skill(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<UseAttackSkill>>,
    query_entity: Query<(&BotCombatTarget, &SkillList, SkillCasterBundle)>,
    query_target: Query<SkillTargetBundle>,
    query_command: Query<(&Command, &NextCommand)>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Some(now) = time.last_update() else {
        return;
    };

    for (&Actor(entity), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                let Ok((bot_combat_target, skill_list, skill_caster)) = query_entity.get(entity) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let Ok(skill_target) = query_target.get(bot_combat_target.entity) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let Some(active_skill_page) = skill_list.pages.get(1) else {
                    *state = ActionState::Failure;
                    continue;
                };

                *state = ActionState::Failure;

                for skill_id in active_skill_page.skills.iter().filter_map(|x| x.as_ref()) {
                    if let Some(skill_data) = game_data.skills.get_skill(*skill_id) {
                        if skill_can_use(now, &game_data, &skill_caster, skill_data)
                            && skill_can_target_entity(&skill_caster, &skill_target, skill_data)
                        {
                            commands.entity(entity).insert(
                                NextCommand::with_cast_skill_target_entity(
                                    *skill_id,
                                    bot_combat_target.entity,
                                    None,
                                ),
                            );
                            *state = ActionState::Executing;
                            break;
                        }
                    }
                }
            }
            ActionState::Executing => {
                let Ok((command, next_command)) = query_command.get(entity) else {
                    *state = ActionState::Failure;
                    continue;
                };

                // Wait until we are not casting any skills
                if !matches!(command.command, CommandData::CastSkill { .. })
                    && !matches!(next_command.command, Some(CommandData::CastSkill { .. }))
                {
                    *state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
