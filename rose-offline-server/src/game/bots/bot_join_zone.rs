use bevy::prelude::{Commands, Component, Query, ResMut, With};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};
use rose_game_common::components::CharacterInfo;

use crate::game::{
    bots::IDLE_DURATION,
    bundles::client_entity_join_zone,
    components::{ClientEntity, ClientEntityType, Command, Position},
    resources::ClientEntityList,
};

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct IsTeleporting {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct JoinZone;

pub fn score_is_teleporting(
    mut query: Query<(&IsTeleporting, &Actor, &mut Score)>,
    query_entity: Query<(Option<&ClientEntity>, Option<&JoinZone>), With<CharacterInfo>>,
) {
    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((client_entity, joining_zone)) = query_entity.get(entity) else {
            continue;
        };

        if client_entity.is_none() || joining_zone.is_some() {
            score.set(scorer.score);
        }
    }
}

pub fn action_join_zone(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<JoinZone>>,
    query_entity: Query<(&Command, &Position, Option<&ClientEntity>)>,
    mut client_entity_list: ResMut<ClientEntityList>,
) {
    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok((command, position, client_entity)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                if client_entity.is_none()
                    && client_entity_join_zone(
                        &mut commands,
                        &mut client_entity_list,
                        entity,
                        ClientEntityType::Character,
                        position,
                    )
                    .is_ok()
                {
                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if client_entity.is_some() && command.is_stop_for(IDLE_DURATION) {
                    // Wait until have a ClientEntity component and we are idle
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
