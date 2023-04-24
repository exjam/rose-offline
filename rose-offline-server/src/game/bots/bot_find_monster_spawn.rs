use bevy::{
    math::Vec3Swizzles,
    prelude::{Commands, Component, Query, Res, With, Without},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState},
    thinker::Actor,
};

use rose_game_common::components::Level;

use crate::game::{
    components::{ClientEntity, Command, Dead, NextCommand, Position},
    GameData,
};

use super::IDLE_DURATION;

#[derive(Debug, Default, Clone, Component, ActionBuilder)]
pub struct FindMonsterSpawns;

pub fn action_find_monster_spawn(
    mut commands: Commands,
    mut query: Query<(&Actor, &mut ActionState), With<FindMonsterSpawns>>,
    query_entity: Query<(&Command, &Level, &Position), (With<ClientEntity>, Without<Dead>)>,
    game_data: Res<GameData>,
) {
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

                // Sort by distance
                potential_spawns.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap());

                // Take the first 5 closest spawns
                potential_spawns.truncate(5);

                // Choose one with smallest level difference
                potential_spawns.sort_by(|lhs, rhs| lhs.1.cmp(&rhs.1));

                if let Some((_, _, index)) = potential_spawns.first() {
                    if let Some(spawn_point) = zone_data.monster_spawns.get(*index) {
                        commands.entity(entity).insert(NextCommand::with_move(
                            spawn_point.position,
                            None,
                            None,
                        ));
                        *state = ActionState::Success;
                        continue;
                    }
                }

                *state = ActionState::Failure;
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
