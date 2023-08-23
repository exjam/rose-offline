use bevy::{
    ecs::prelude::{Commands, Entity, Query, Res, ResMut},
    time::Time,
};

use rose_data::NpcId;

use crate::game::{
    bundles::MonsterBundle,
    components::{MonsterSpawnPoint, Position, SpawnOrigin, Team},
    resources::{ClientEntityList, GameData, ZoneList},
};

pub fn monster_spawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut MonsterSpawnPoint, &Position)>,
    time: Res<Time>,
    mut client_entity_list: ResMut<ClientEntityList>,
    game_data: Res<GameData>,
    zone_list: Res<ZoneList>,
) {
    query.for_each_mut(
        |(spawn_point_entity, mut spawn_point, spawn_point_position)| {
            if !zone_list.get_monster_spawns_enabled(spawn_point_position.zone_id) {
                return;
            }

            let spawn_point = &mut *spawn_point;
            spawn_point.time_since_last_check += time.delta();
            if spawn_point.time_since_last_check < spawn_point.interval {
                return;
            }
            spawn_point.time_since_last_check -= spawn_point.interval;

            let live_count = spawn_point.num_alive_monsters;
            if live_count >= spawn_point.limit_count {
                spawn_point.current_tactics_value =
                    spawn_point.current_tactics_value.saturating_sub(1);
                return;
            }

            let regen_value = ((spawn_point.limit_count * 2 - live_count)
                * spawn_point.current_tactics_value
                * 50)
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

            for (npc_id, count) in spawn_queue {
                for _ in 0..count {
                    if MonsterBundle::spawn(
                        &mut commands,
                        &mut client_entity_list,
                        &game_data,
                        npc_id,
                        spawn_point_zone,
                        SpawnOrigin::MonsterSpawnPoint(spawn_point_entity, spawn_point_position),
                        spawn_range,
                        Team::default_monster(),
                        None,
                        None,
                    )
                    .is_some()
                    {
                        spawn_point.num_alive_monsters += 1;
                    }
                }
            }
        },
    );
}
