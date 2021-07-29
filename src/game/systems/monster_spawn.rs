use legion::{system, systems::CommandBuffer, Entity};
use nalgebra::Point3;
use rand::Rng;

use crate::{
    data::NpcId,
    game::{
        bundles::{client_entity_join_zone, create_monster_entity},
        components::{
            ClientEntityType, Command, DamageSources, HealthPoints, Level, MonsterSpawnPoint,
            MoveSpeed, NextCommand, Npc, NpcAi, Position, SpawnOrigin, Team,
        },
        resources::{ClientEntityList, GameData, ServerTime},
    },
};

#[system(for_each)]
pub fn monster_spawn(
    cmd: &mut CommandBuffer,
    spawn_point_entity: &Entity,
    spawn_point: &mut MonsterSpawnPoint,
    spawn_point_position: &Position,
    #[resource] server_time: &ServerTime,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
) {
    spawn_point.time_since_last_check += server_time.delta;
    if spawn_point.time_since_last_check < spawn_point.interval {
        return;
    }
    spawn_point.time_since_last_check -= spawn_point.interval;

    let live_count = spawn_point.num_alive_monsters;
    if live_count >= spawn_point.limit_count {
        spawn_point.current_tactics_value = spawn_point.current_tactics_value.saturating_sub(1);
        return;
    }

    let regen_value =
        ((spawn_point.limit_count * 2 - live_count) * spawn_point.current_tactics_value * 50)
            / (spawn_point.limit_count * spawn_point.tactic_points);

    let mut spawn_queue: Vec<(NpcId, usize)> = Vec::new();
    match regen_value {
        0..=10 => {
            // Spawn basic[0]
            spawn_point.current_tactics_value += 12;
            if let Some((id, count)) = spawn_point.basic_spawns.get(0) {
                spawn_queue.push((*id, *count))
            }
        }
        11..=15 => {
            // Spawn basic[0] - 2, basic[1]
            spawn_point.current_tactics_value += 15;
            if let Some((id, count)) = spawn_point.basic_spawns.get(0) {
                spawn_queue.push((*id, count.saturating_sub(2)))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(1) {
                spawn_queue.push((*id, *count))
            }
        }
        16..=25 => {
            // Spawn basic[2]
            spawn_point.current_tactics_value += 12;
            if let Some((id, count)) = spawn_point.basic_spawns.get(2) {
                spawn_queue.push((*id, *count))
            }
        }
        26..=30 => {
            // Spawn basic[0] - 1, basic[2]
            spawn_point.current_tactics_value += 15;
            if let Some((id, count)) = spawn_point.basic_spawns.get(0) {
                spawn_queue.push((*id, count.saturating_sub(1)))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(2) {
                spawn_queue.push((*id, *count))
            }
        }
        31..=40 => {
            // Spawn basic[3]
            spawn_point.current_tactics_value += 12;
            if let Some((id, count)) = spawn_point.basic_spawns.get(3) {
                spawn_queue.push((*id, *count))
            }
        }
        41..=50 => {
            // Spawn basic[1], basic[2] - 2
            spawn_point.current_tactics_value += 12;
            if let Some((id, count)) = spawn_point.basic_spawns.get(1) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(2) {
                spawn_queue.push((*id, count.saturating_sub(1)))
            }
        }
        51..=65 => {
            // Spawn basic[2], basic[3] - 2
            spawn_point.current_tactics_value += 20;
            if let Some((id, count)) = spawn_point.basic_spawns.get(2) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(3) {
                spawn_queue.push((*id, count.saturating_sub(2)))
            }
        }
        66..=73 => {
            // Spawn basic[3], basic[4]
            spawn_point.current_tactics_value += 15;
            if let Some((id, count)) = spawn_point.basic_spawns.get(3) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(4) {
                spawn_queue.push((*id, *count))
            }
        }
        74..=85 => {
            // Spawn basic[0], basic[4] - 2, tactics[0] - 1
            spawn_point.current_tactics_value += 15;
            if let Some((id, count)) = spawn_point.basic_spawns.get(0) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.basic_spawns.get(4) {
                spawn_queue.push((*id, count.saturating_sub(2)))
            }
            if let Some((id, count)) = spawn_point.tactic_spawns.get(0) {
                spawn_queue.push((*id, count.saturating_sub(1)))
            }
        }
        86..=92 => {
            // Spawn basic[1], tactics[0], tactics[1]
            spawn_point.current_tactics_value = 1;
            if let Some((id, count)) = spawn_point.basic_spawns.get(1) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.tactic_spawns.get(0) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.tactic_spawns.get(1) {
                spawn_queue.push((*id, *count))
            }
        }
        _ => {
            // Spawn basic[4], tactics[0] + 1, tactics[1]
            spawn_point.current_tactics_value = 7;
            if let Some((id, count)) = spawn_point.basic_spawns.get(4) {
                spawn_queue.push((*id, *count))
            }
            if let Some((id, count)) = spawn_point.tactic_spawns.get(0) {
                spawn_queue.push((*id, count + 1))
            }
            if let Some((id, count)) = spawn_point.tactic_spawns.get(1) {
                spawn_queue.push((*id, *count))
            }
        }
    }

    if spawn_point.current_tactics_value > 500 {
        spawn_point.current_tactics_value = 500;
    }

    let spawn_point_zone = spawn_point_position.zone_id;
    let spawn_point_position = spawn_point_position.position;
    let spawn_range = (spawn_point.range * 100) as i32;

    for (id, count) in spawn_queue {
        for _ in 0..count {
            let npc_data = game_data.npcs.get_npc(id);
            let ability_values = game_data.ability_value_calculator.calculate_npc(id);

            if let (Some(npc_data), Some(ability_values)) = (npc_data, ability_values) {
                let npc_ai = Some(npc_data.ai_file_index)
                    .filter(|ai_file_index| *ai_file_index != 0)
                    .map(|ai_file_index| NpcAi::new(ai_file_index as usize));

                let damage_sources = Some(ability_values.max_damage_sources)
                    .filter(|max_damage_sources| *max_damage_sources > 0)
                    .map(DamageSources::new);
                let health_points = HealthPoints::new(ability_values.max_health as u32);
                let level = Level::new(ability_values.level as u32);
                let move_speed = MoveSpeed::new(ability_values.walk_speed as f32);

                let position = Position::new(
                    Point3::new(
                        spawn_point_position.x
                            + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                        spawn_point_position.y
                            + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                        0.0,
                    ),
                    spawn_point_zone,
                );

                let entity = cmd.push(());

                create_monster_entity(
                    cmd,
                    &entity,
                    ability_values,
                    Command::default(),
                    damage_sources,
                    health_points,
                    level,
                    game_data.npcs.get_npc_motions(id),
                    move_speed,
                    NextCommand::default(),
                    Npc::new(id, 0),
                    npc_ai,
                    position.clone(),
                    SpawnOrigin::MonsterSpawnPoint(*spawn_point_entity, spawn_point_position),
                    Team::default_monster(),
                );

                client_entity_join_zone(
                    cmd,
                    client_entity_list,
                    &entity,
                    ClientEntityType::Monster,
                    &position,
                )
                .expect("Failed to join monster into zone");

                spawn_point.num_alive_monsters += 1;
            }
        }
    }
}
