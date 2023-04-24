use std::time::Duration;

use bevy::prelude::{Commands, Component, Query, With, Without};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use crate::game::components::{
    AbilityValues, ClientEntity, Command, Dead, HealthPoints, NextCommand,
};

const DEAD_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ShouldSitRecoverHp {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct SitRecoverHp;

pub fn score_should_sit_recover_hp(
    mut query: Query<(&ShouldSitRecoverHp, &Actor, &mut Score)>,
    query_entity: Query<
        (&Command, &AbilityValues, &HealthPoints),
        (With<ClientEntity>, Without<Dead>),
    >,
) {
    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((command, ability_values, health_points)) = query_entity.get(entity) else {
            continue;
        };

        if command.is_sit() {
            // When we sit, we might aswell wait until full health
            if health_points.hp < ability_values.get_max_health() {
                score.set(scorer.score);
            }
        } else {
            // Start sit when < 40% hp
            let health_percent = health_points.hp as f32 / ability_values.get_max_health() as f32;
            if health_percent < 0.4 {
                score.set(scorer.score);
            }
        }
    }
}

pub fn action_sit_recover_hp(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<SitRecoverHp>>,
) {
    for (&Actor(entity), mut state) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                commands.entity(entity).insert(NextCommand::with_sitting());
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                // This task will execute until it is cancelled
            }
            ActionState::Cancelled => {
                commands.entity(entity).insert(NextCommand::with_standing());
                *state = ActionState::Success;
            }
            _ => {}
        }
    }
}
