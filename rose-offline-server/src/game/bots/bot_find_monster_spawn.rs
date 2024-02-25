use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Query, Res, Vec3, With},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState},
    thinker::Actor,
};

use rand::{seq::SliceRandom, Rng};
use rose_game_common::components::Level;

use crate::game::{
    components::{Command, NextCommand, Position},
    GameData,
};

use super::{BotQueryFilterAlive, IDLE_DURATION};

#[derive(Debug, Default, Clone, Component, ActionBuilder)]
pub struct FindMonsterSpawns;

pub fn action_find_monster_spawn(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<FindMonsterSpawns>>,
    query_entity: Query<(&Command, &Level, &Position), BotQueryFilterAlive>,
    game_data: Res<GameData>,
) {
    let mut rng = rand::thread_rng();

    for (&Actor(entity), mut state) in query.iter_mut() {
        let Ok((command, level, position)) = query_entity.get(entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                let Some(zone_data) = game_data.zones.get_zone(position.zone_id) else {
                    *state = ActionState::Failure;
                    continue;
                };

                let mut potential_spawns = Vec::with_capacity(zone_data.monster_spawns.len());

                for (i, spawn) in zone_data.monster_spawns.iter().enumerate() {
                    let mut total_level = 0;
                    let mut num_monsters = 0;

                    for npc_id in spawn.basic_spawns.iter().map(|(npc_id, _)| *npc_id) {
                        if let Some(npc_data) = game_data.npcs.get_npc(npc_id) {
                            total_level += npc_data.level;
                            num_monsters += 1;
                        }
                    }

                    for npc_id in spawn.tactic_spawns.iter().map(|(npc_id, _)| *npc_id) {
                        if let Some(npc_data) = game_data.npcs.get_npc(npc_id) {
                            total_level += npc_data.level;
                            num_monsters += 1;
                        }
                    }

                    let average_level = total_level / num_monsters;
                    let level_difference = (level.level as i32 - average_level).abs();
                    let distance = position.position.xy().distance_squared(spawn.position.xy());

                    potential_spawns.push((distance, level_difference, i));
                }

                if potential_spawns.is_empty() {
                    *state = ActionState::Failure;
                    continue;
                }

                // Take the 10 spawns with smallest level delta to our character
                potential_spawns.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));
                potential_spawns.truncate(10);

                // Then take the 5 closest
                potential_spawns.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap());
                potential_spawns.truncate(5);

                // Choose one randomly
                let Some(spawn_point) = potential_spawns
                    .choose(&mut rng)
                    .and_then(|(_, _, index)| zone_data.monster_spawns.get(*index))
                else {
                    *state = ActionState::Failure;
                    continue;
                };

                // Move near the center of spawn
                let range = (spawn_point.range * 100).max(500) as f32;
                commands.entity(entity).insert(NextCommand::with_move(
                    spawn_point.position
                        + Vec3::new(
                            rng.gen_range(-range..range),
                            rng.gen_range(-range..range),
                            0.0,
                        ),
                    None,
                    None,
                ));
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if command.is_stop_for(IDLE_DURATION) {
                    *state = ActionState::Success;
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Success;
            }
            _ => {}
        }
    }
}
