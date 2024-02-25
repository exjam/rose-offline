use std::time::{Duration, Instant};

use bevy::prelude::{Commands, Component, Entity, Query, With};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use crate::game::{
    bots::BotCombatTarget,
    components::{Command, DamageSources, HealthPoints, NextCommand, Team},
};

use super::BotQueryFilterAlive;

const RECENT_ATTACK_TIME: Duration = Duration::from_secs(5);

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct ThreatIsNotTarget {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct AttackThreat;

fn find_highest_damage_source(damage_sources: &DamageSources) -> Option<Entity> {
    let now = Instant::now();

    let mut highest_damage_source = None;
    for damage_source in damage_sources.damage_sources.iter() {
        if now - damage_source.last_damage_time > RECENT_ATTACK_TIME {
            continue;
        }

        if highest_damage_source.map_or(true, |(total_damage, _)| {
            total_damage < damage_source.total_damage
        }) {
            highest_damage_source = Some((damage_source.total_damage, damage_source.entity));
        }
    }

    highest_damage_source.map(|(_, entity)| entity)
}

pub fn score_threat_is_not_target(
    mut query: Query<(&ThreatIsNotTarget, &Actor, &mut Score)>,
    query_entity: Query<
        (Option<&BotCombatTarget>, &Command, &DamageSources, &Team),
        BotQueryFilterAlive,
    >,
    query_target: Query<(&Team, &HealthPoints)>,
) {
    let now = Instant::now();

    for (scorer, &Actor(entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok((bot_combat_target, command, damage_sources, team)) = query_entity.get(entity)
        else {
            continue;
        };

        if command.is_dead() {
            // Cannot fight whilst dead
            continue;
        }

        let mut highest_damage_source = None;
        let mut bot_combat_target_damage = 0;

        for damage_source in damage_sources.damage_sources.iter() {
            if now - damage_source.last_damage_time > RECENT_ATTACK_TIME {
                continue;
            }

            if highest_damage_source.map_or(true, |(total_damage, _)| {
                total_damage < damage_source.total_damage
            }) {
                highest_damage_source = Some((damage_source.total_damage, damage_source.entity));
            }

            if bot_combat_target.map_or(false, |bot_combat_target| {
                bot_combat_target.entity == damage_source.entity
            }) {
                bot_combat_target_damage = damage_source.total_damage;
            }
        }

        let Some((highest_damage, highest_damage_source_entity)) = highest_damage_source else {
            continue;
        };

        if bot_combat_target.map_or(false, |bot_combat_target| {
            bot_combat_target.entity == highest_damage_source_entity
        }) {
            // We are already targeting the highest damage source
            continue;
        }

        if bot_combat_target_damage * 10 >= highest_damage * 8 {
            // Current target damage is within 80% of highest, do not switch target
            continue;
        }

        if let Ok((target_team, target_health_points)) =
            query_target.get(highest_damage_source_entity)
        {
            // Check the target is still valid before starting switch
            if target_team.id != team.id && target_health_points.hp > 0 {
                score.set(scorer.score);
            }
        }
    }
}

pub fn action_attack_threat(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<AttackThreat>>,
    query_entity: Query<(&Team, &DamageSources)>,
    query_target: Query<(&Team, &HealthPoints)>,
) {
    let now = Instant::now();

    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok((team, damage_sources)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                let mut highest_damage_source = None;
                for damage_source in damage_sources.damage_sources.iter() {
                    if now - damage_source.last_damage_time > RECENT_ATTACK_TIME {
                        continue;
                    }

                    if highest_damage_source.map_or(true, |(total_damage, _)| {
                        total_damage < damage_source.total_damage
                    }) {
                        highest_damage_source =
                            Some((damage_source.total_damage, damage_source.entity));
                    }
                }

                if let Some((_, highest_damage_source_entity)) = highest_damage_source {
                    if let Ok((target_team, target_health_points)) =
                        query_target.get(highest_damage_source_entity)
                    {
                        if target_team.id != team.id && target_health_points.hp > 0 {
                            commands
                                .entity(entity)
                                .insert(BotCombatTarget {
                                    entity: highest_damage_source_entity,
                                })
                                .insert(NextCommand::with_attack(highest_damage_source_entity));

                            *state = ActionState::Success;
                            continue;
                        }
                    }
                }

                *state = ActionState::Failure;
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
