use std::time::Duration;

use bevy::prelude::{Commands, Component, EventWriter, Query, With};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use crate::game::{
    bots::IDLE_DURATION,
    components::{Command, Dead},
    events::{ReviveEvent, RevivePosition},
};

use super::BotCombatTarget;

const DEAD_DURATION: Duration = Duration::from_secs(10);

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct IsDead {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct ReviveCurrentZone;

pub fn score_is_dead(
    mut query: Query<(&IsDead, &Actor, &mut Score)>,
    query_entity: Query<(&Command, Option<&ReviveCurrentZone>), With<Dead>>,
) {
    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((command, is_reviving)) = query_entity.get(entity) else {
            continue;
        };

        if command.is_dead() || is_reviving.is_some() {
            score.set(scorer.score);
        }
    }
}

pub fn action_revive_current_zone(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<ReviveCurrentZone>>,
    query_entity: Query<&Command>,
    mut revive_events: EventWriter<ReviveEvent>,
) {
    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok(command) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                if command.is_dead_for(DEAD_DURATION) {
                    commands.entity(entity).remove::<BotCombatTarget>();
                    revive_events.send(ReviveEvent {
                        entity,
                        position: RevivePosition::CurrentZone,
                    });
                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if command.is_stop_for(IDLE_DURATION) {
                    // Wait until we are idle
                    *state = ActionState::Success;
                    continue;
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
