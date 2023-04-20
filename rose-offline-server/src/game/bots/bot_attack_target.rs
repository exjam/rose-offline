use bevy::prelude::{Commands, Component, Query, With};

use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use rose_game_common::components::AbilityValues;

use crate::game::{
    bots::{BotCombatTarget, IDLE_DURATION},
    components::{Command, HealthPoints, NextCommand},
};

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ShouldAttackTarget {
    pub min_score: f32,
    pub max_score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct ActionAttackTarget;

pub fn score_should_attack_target(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut Score, &ShouldAttackTarget)>,
    query_entity: Query<(&Command, &BotCombatTarget)>,
    query_target: Query<(&AbilityValues, &HealthPoints)>,
) {
    for (&Actor(entity), mut score, should_attack_target) in query.iter_mut() {
        score.set(0.0);

        let Ok((command, bot_combat_target)) = query_entity.get(entity) else {
            continue;
        };

        if command.is_dead() {
            // Cannot fight whilst dead
            continue;
        }

        let Ok((target_ability_values, target_health_points)) = query_target.get(bot_combat_target.entity) else {
            commands.entity(entity).remove::<BotCombatTarget>(); // Target no longer exists
            continue;
        };

        let weight = 1.0
            - (target_health_points.hp as f32 / target_ability_values.max_health as f32)
                .clamp(0.0, 1.0);
        let score_value = weight
            * (should_attack_target.max_score - should_attack_target.min_score)
            + should_attack_target.min_score;
        score.set(score_value);
    }
}

pub fn action_attack_target(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<ActionAttackTarget>>,
    query_entity: Query<(&BotCombatTarget, &Command)>,
    query_target: Query<&HealthPoints>,
) {
    for (&Actor(entity), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                let Ok((bot_combat_target, command)) = query_entity.get(entity) else {
                    *state = ActionState::Failure;
                    continue;
                };

                if query_target
                    .get(bot_combat_target.entity)
                    .map_or(false, |target_health_points| target_health_points.hp > 0)
                {
                    // Ensure we are attacking target
                    if !command.is_attack_target(bot_combat_target.entity) {
                        commands
                            .entity(entity)
                            .insert(NextCommand::with_attack(bot_combat_target.entity));
                    }
                } else {
                    // Wait until attack is complete
                    if command.is_stop_for(IDLE_DURATION) {
                        commands.entity(entity).remove::<BotCombatTarget>();
                        *state = ActionState::Success;
                    }
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
