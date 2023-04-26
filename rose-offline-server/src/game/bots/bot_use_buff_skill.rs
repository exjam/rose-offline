use bevy::{
    ecs::query::WorldQuery,
    prelude::{Commands, Component, Query, Res, With},
    time::Time,
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};
use rose_data::{SkillTargetFilter, SkillType};

use crate::game::{
    bundles::{skill_can_use, SkillCasterBundle},
    components::{Command, CommandData, NextCommand, SkillList, StatusEffects},
    GameData,
};

use super::{BotQueryFilterAlive, BotQueryFilterAliveNoTarget};

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ShouldUseBuffSkill {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct UseBuffSkill;

#[derive(WorldQuery)]
pub struct BotQuery<'w> {
    skill_list: &'w SkillList,
    status_effects: &'w StatusEffects,
    skill_caster: SkillCasterBundle<'w>,
    is_using_skill: Option<&'w UseBuffSkill>,
}

pub fn score_should_use_buff_skill(
    mut query: Query<(&ShouldUseBuffSkill, &Actor, &mut Score)>,
    query_entity: Query<BotQuery, BotQueryFilterAliveNoTarget>,
    game_data: Res<GameData>,
    time: Res<Time>,
) {
    let Some(now) = time.last_update() else {
        return;
    };

    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok(bot) = query_entity.get(entity) else {
            continue;
        };

        if bot.is_using_skill.is_some() {
            score.set(scorer.score);
            continue;
        }

        let Some(mana_points) = bot.skill_caster.mana_points else {
            continue;
        };

        if (mana_points.mp as f32 / bot.skill_caster.ability_values.get_max_mana() as f32) < 0.25 {
            continue;
        }

        let Some(active_skill_page) = bot.skill_list.pages.get(1) else {
            continue;
        };

        for skill_data in active_skill_page
            .skills
            .iter()
            .filter_map(|skill_slot| skill_slot.as_ref())
            .filter_map(|skill_id| game_data.skills.get_skill(*skill_id))
        {
            if (skill_data.status_effects[0].is_none() && skill_data.status_effects[1].is_none())
                || !matches!(
                    skill_data.skill_type,
                    SkillType::SelfBoundDuration
                        | SkillType::SelfStateDuration
                        | SkillType::TargetBoundDuration
                        | SkillType::TargetStateDuration
                        | SkillType::SelfBound
                        | SkillType::TargetBound
                )
                || !matches!(
                    skill_data.target_filter,
                    SkillTargetFilter::OnlySelf
                        | SkillTargetFilter::Allied
                        | SkillTargetFilter::Group
                )
            {
                // Only looking for buffs which can apply to self
                continue;
            }

            let already_has_status_effect = skill_data
                .status_effects
                .iter()
                .filter_map(|x| *x)
                .filter_map(|status_effect_id| {
                    game_data.status_effects.get_status_effect(status_effect_id)
                })
                .any(|status_effect_data| {
                    bot.status_effects
                        .get_status_effect_value(status_effect_data.status_effect_type)
                        .is_some()
                });

            if already_has_status_effect {
                continue;
            }

            if skill_can_use(now, &game_data, &bot.skill_caster, skill_data) {
                score.set(scorer.score);
                break;
            }
        }
    }
}

pub fn action_use_buff_skill(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<UseBuffSkill>>,
    query_entity: Query<BotQuery, BotQueryFilterAlive>,
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
                let Ok(bot) = query_entity.get(entity) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let Some(active_skill_page) = bot.skill_list.pages.get(1) else {
                    *state = ActionState::Failure;
                    continue;
                };

                *state = ActionState::Failure;

                for skill_data in active_skill_page
                    .skills
                    .iter()
                    .filter_map(|skill_slot| skill_slot.as_ref())
                    .filter_map(|skill_id| game_data.skills.get_skill(*skill_id))
                {
                    if (skill_data.status_effects[0].is_none()
                        && skill_data.status_effects[1].is_none())
                        || !matches!(
                            skill_data.skill_type,
                            SkillType::SelfBoundDuration
                                | SkillType::SelfStateDuration
                                | SkillType::TargetBoundDuration
                                | SkillType::TargetStateDuration
                                | SkillType::SelfBound
                                | SkillType::TargetBound
                        )
                        || !matches!(
                            skill_data.target_filter,
                            SkillTargetFilter::OnlySelf
                                | SkillTargetFilter::Allied
                                | SkillTargetFilter::Group
                        )
                    {
                        // Only looking for buffs which can apply to self
                        continue;
                    }

                    let already_has_status_effect = skill_data
                        .status_effects
                        .iter()
                        .filter_map(|x| *x)
                        .filter_map(|status_effect_id| {
                            game_data.status_effects.get_status_effect(status_effect_id)
                        })
                        .any(|status_effect_data| {
                            bot.status_effects
                                .get_status_effect_value(status_effect_data.status_effect_type)
                                .is_some()
                        });

                    if already_has_status_effect {
                        continue;
                    }

                    if skill_can_use(now, &game_data, &bot.skill_caster, skill_data) {
                        commands
                            .entity(entity)
                            .insert(NextCommand::with_cast_skill_target_self(
                                skill_data.id,
                                None,
                            ));
                        *state = ActionState::Executing;
                        break;
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
