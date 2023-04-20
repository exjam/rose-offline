use arrayvec::ArrayVec;
use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Entity, Query, Res, With},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};
use rand::seq::SliceRandom;

use crate::game::{
    bots::IDLE_DURATION,
    components::{ClientEntityType, Command, HealthPoints, NextCommand, Position, Team},
    resources::ClientEntityList,
};

use super::BotCombatTarget;

const NEAREST_TARGET_SEARCH_DISTANCE: f32 = 2000.0f32;

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct FindNearbyTarget {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct AttackRandomNearbyTarget;

pub fn score_find_nearby_target(
    mut query: Query<(&FindNearbyTarget, &Actor, &mut Score)>,
    query_entity: Query<(&Command, &Position, &Team, Option<&BotCombatTarget>)>,
    query_target: Query<(&Team, &HealthPoints)>,
    client_entity_list: Res<ClientEntityList>,
) {
    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((command, position, team, bot_combat_target)) = query_entity.get(entity) else {
            continue;
        };

        if command.is_dead() {
            // Cannot fight whilst dead
            continue;
        }

        if bot_combat_target.is_some() {
            // We already have a target
            continue;
        }

        let Some(zone_entities) =
            client_entity_list.get_zone(position.zone_id) else {
                continue;
            };

        if zone_entities
            .iter_entity_type_within_distance(
                position.position.xy(),
                NEAREST_TARGET_SEARCH_DISTANCE,
                &[ClientEntityType::Character, ClientEntityType::Monster],
            )
            .any(|(nearby_entity, _)| {
                query_target.get(nearby_entity).ok().map_or(
                    false,
                    |(nearby_team, nearby_health_points)| {
                        nearby_team.id != team.id && nearby_health_points.hp > 0
                    },
                )
            })
        {
            score.set(scorer.score);
        }
    }
}

pub fn action_attack_random_nearby_target(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<AttackRandomNearbyTarget>>,
    query_entity: Query<(&Command, &Position, &Team)>,
    query_target: Query<(&Team, &HealthPoints)>,
    client_entity_list: Res<ClientEntityList>,
) {
    let mut rng = rand::thread_rng();

    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok((command, position, team)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                let Some(zone_entities) =
                    client_entity_list.get_zone(position.zone_id) else {
                        continue;
                    };

                // Find the 10 nearest living enemies
                let mut nearest_targets: ArrayVec<(f32, Entity), 10> = ArrayVec::new();
                for (nearby_entity, nearby_position) in zone_entities
                    .iter_entity_type_within_distance(
                        position.position.xy(),
                        NEAREST_TARGET_SEARCH_DISTANCE,
                        &[ClientEntityType::Character, ClientEntityType::Monster],
                    )
                {
                    if query_target.get(nearby_entity).ok().map_or(
                        false,
                        |(nearby_team, nearby_health_points)| {
                            nearby_team.id != team.id && nearby_health_points.hp > 0
                        },
                    ) {
                        let distance = position
                            .position
                            .xy()
                            .distance_squared(nearby_position.xy());

                        for (index, (nearest_distance, _)) in nearest_targets.iter().enumerate() {
                            if distance < *nearest_distance {
                                if nearest_targets.is_full() {
                                    let last = nearest_targets.len() - 1;
                                    nearest_targets.remove(last);
                                }

                                nearest_targets.insert(index, (distance, nearby_entity));
                                break;
                            }
                        }

                        if nearest_targets.is_empty() {
                            nearest_targets.push((distance, nearby_entity));
                        }
                    }
                }

                // Choose random target to attack
                if let Some(&(_, nearest_entity)) = nearest_targets.choose(&mut rng) {
                    commands
                        .entity(entity)
                        .insert(NextCommand::with_attack(nearest_entity))
                        .insert(BotCombatTarget {
                            entity: nearest_entity,
                        });

                    *state = ActionState::Executing;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Executing => {
                if command.is_stop_for(IDLE_DURATION) {
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
