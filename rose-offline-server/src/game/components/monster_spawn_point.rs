use bevy::ecs::prelude::Component;
use std::time::Duration;

use rose_data::{NpcId, ZoneMonsterSpawnPoint};

#[derive(Component)]
pub struct MonsterSpawnPoint {
    pub basic_spawns: Vec<(NpcId, usize)>,
    pub tactic_spawns: Vec<(NpcId, usize)>,
    pub interval: Duration,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,

    pub time_since_last_check: Duration,
    pub current_tactics_value: u32,
    pub num_alive_monsters: u32,
}

impl From<&ZoneMonsterSpawnPoint> for MonsterSpawnPoint {
    fn from(spawn_point: &ZoneMonsterSpawnPoint) -> Self {
        Self {
            basic_spawns: spawn_point.basic_spawns.clone(),
            tactic_spawns: spawn_point.tactic_spawns.clone(),
            interval: Duration::from_secs(spawn_point.interval as u64),
            limit_count: spawn_point.limit_count,
            range: spawn_point.range,
            tactic_points: spawn_point.tactic_points,

            time_since_last_check: Duration::from_millis(0),
            current_tactics_value: 0,
            num_alive_monsters: 0,
        }
    }
}
