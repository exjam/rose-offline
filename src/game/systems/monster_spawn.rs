use nalgebra::Point3;
use rand::Rng;

use crate::game::{
    components::{Monster, MonsterSpawnPoint, Position},
    resources::{ClientEntityList, DeltaTime},
};
use legion::{system, systems::CommandBuffer};

#[system(for_each)]
pub fn monster_spawn(
    cmd: &mut CommandBuffer,
    spawn_point: &mut MonsterSpawnPoint,
    spawn_point_position: &Position,
    #[resource] delta_time: &DeltaTime,
    #[resource] client_entity_list: &mut ClientEntityList,
) {
    spawn_point.time_since_last_check += delta_time.delta;
    if spawn_point.time_since_last_check < spawn_point.interval {
        return;
    }
    spawn_point.time_since_last_check -= spawn_point.interval;

    let live_count = spawn_point.monsters.len() as u32;
    if live_count >= spawn_point.limit_count {
        spawn_point.current_tactics_value = spawn_point.current_tactics_value.saturating_sub(1);
        return;
    }

    let regen_value =
        ((spawn_point.limit_count * 2 - live_count) * spawn_point.current_tactics_value * 50)
            / (spawn_point.limit_count * spawn_point.tactic_points);

    let mut spawn_queue = Vec::new();
    match regen_value {
        0..=10 => {
            // Spawn basic[0]
            spawn_point.current_tactics_value += 12;
            spawn_point
                .basic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        11..=15 => {
            // Spawn basic[0] - 2, basic[1]
            spawn_point.current_tactics_value += 15;
            spawn_point
                .basic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(2))));
            spawn_point
                .basic_spawns
                .get(1)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        16..=25 => {
            // Spawn basic[2]
            spawn_point.current_tactics_value += 12;
            spawn_point
                .basic_spawns
                .get(2)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        26..=30 => {
            // Spawn basic[0] - 1, basic[2]
            spawn_point.current_tactics_value += 15;
            spawn_point
                .basic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(1))));
            spawn_point
                .basic_spawns
                .get(2)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        31..=40 => {
            // Spawn basic[3]
            spawn_point.current_tactics_value += 12;
            spawn_point
                .basic_spawns
                .get(3)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        41..=50 => {
            // Spawn basic[1], basic[2] - 2
            spawn_point.current_tactics_value += 12;
            spawn_point
                .basic_spawns
                .get(1)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .basic_spawns
                .get(2)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(1))));
        }
        51..=65 => {
            // Spawn basic[2], basic[3] - 2
            spawn_point.current_tactics_value += 20;
            spawn_point
                .basic_spawns
                .get(2)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .basic_spawns
                .get(3)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(2))));
        }
        66..=73 => {
            // Spawn basic[3], basic[4]
            spawn_point.current_tactics_value += 15;
            spawn_point
                .basic_spawns
                .get(3)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .basic_spawns
                .get(4)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        74..=85 => {
            // Spawn basic[0], basic[4] - 2, tactics[0] - 1
            spawn_point.current_tactics_value += 15;
            spawn_point
                .basic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .basic_spawns
                .get(4)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(2))));
            spawn_point
                .tactic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count.saturating_sub(1))));
        }
        86..=92 => {
            // Spawn basic[1], tactics[0], tactics[1]
            spawn_point.current_tactics_value = 1;
            spawn_point
                .basic_spawns
                .get(1)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .tactic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .tactic_spawns
                .get(1)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
        }
        _ => {
            // Spawn basic[4], tactics[0] + 1, tactics[1]
            spawn_point.current_tactics_value = 7;
            spawn_point
                .basic_spawns
                .get(4)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
            spawn_point
                .tactic_spawns
                .get(0)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count + 1)));
            spawn_point
                .tactic_spawns
                .get(1)
                .map(|spawn| spawn_queue.push((spawn.id, spawn.count)));
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
            let entity = cmd.push((Monster::new(id), Position::new(position, spawn_point_zone)));
            cmd.add_component(
                entity,
                client_entity_zone.allocate(entity, position).unwrap(),
            );
            spawn_point.monsters.push(entity);
        }
    }
}
