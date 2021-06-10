use legion::{system, systems::CommandBuffer, Entity};
use nalgebra::Point3;
use rand::Rng;

use crate::{
    data::NpcReference,
    game::{
        components::{
            Command, DamageSources, HealthPoints, MonsterSpawnPoint, NextCommand, Npc, NpcAi,
            Position, SpawnOrigin, Team,
        },
        resources::{ClientEntityList, DeltaTime, GameData},
    },
};

#[system(for_each)]
pub fn monster_spawn(
    cmd: &mut CommandBuffer,
    spawn_point_entity: &Entity,
    spawn_point: &mut MonsterSpawnPoint,
    spawn_point_position: &Position,
    #[resource] delta_time: &DeltaTime,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
) {
    spawn_point.time_since_last_check += delta_time.delta;
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

    let mut spawn_queue: Vec<(NpcReference, usize)> = Vec::new();
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

    let spawn_point_zone = spawn_point_position.zone;
    let spawn_point_position = spawn_point_position.position;
    let client_entity_zone = client_entity_list
        .get_zone_mut(spawn_point_zone as usize)
        .unwrap();
    let spawn_range = (spawn_point.range * 100) as i32;

    for (id, count) in spawn_queue {
        for _ in 0..count {
            let position = Point3::new(
                spawn_point_position.x
                    + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                spawn_point_position.y
                    + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                0.0,
            );
            let entity = cmd.push((
                Npc::new(id.0 as u32, 0),
                Position::new(position, spawn_point_zone),
                Team::default_monster(),
                DamageSources::new(),
                SpawnOrigin::MonsterSpawnPoint(*spawn_point_entity, spawn_point_position),
                Command::default(),
                NextCommand::default(),
            ));
            cmd.add_component(
                entity,
                client_entity_zone.allocate(entity, position).unwrap(),
            );

            let ai_file_index = game_data
                .npcs
                .get_npc(id.0)
                .map(|npc_data| npc_data.ai_file_index)
                .unwrap_or(0);
            if ai_file_index != 0 {
                cmd.add_component(entity, NpcAi::new(ai_file_index as usize));
            }

            if let Some(ability_values) = game_data
                .ability_value_calculator
                .calculate_npc(id.0 as usize)
            {
                cmd.add_component(
                    entity,
                    HealthPoints {
                        hp: ability_values.max_health as u32,
                    },
                );
                cmd.add_component(entity, ability_values);
            }

            spawn_point.num_alive_monsters += 1;
        }
    }
}
