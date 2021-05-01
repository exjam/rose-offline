use legion::Entity;
use nalgebra::Point3;
use std::time::Duration;

use crate::game::data::formats::ifo;

pub struct MonsterSpawn {
    pub id: u32,
    pub count: u32,
}

pub struct MonsterSpawnPoint {
    pub basic_spawns: Vec<MonsterSpawn>,
    pub tactic_spawns: Vec<MonsterSpawn>,
    pub interval: Duration,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,

    pub time_since_last_check: Duration,
    pub current_tactics_value: u32,
    pub monsters: Vec<Entity>,
}

impl From<&ifo::MonsterSpawnPoint> for MonsterSpawnPoint {
    fn from(spawn_point: &ifo::MonsterSpawnPoint) -> Self {
        Self {
            basic_spawns: spawn_point
                .basic_spawns
                .iter()
                .map(|x| MonsterSpawn {
                    id: x.id,
                    count: x.count,
                })
                .collect(),
            tactic_spawns: spawn_point
                .tactic_spawns
                .iter()
                .map(|x| MonsterSpawn {
                    id: x.id,
                    count: x.count,
                })
                .collect(),
            interval: Duration::from_secs(spawn_point.interval as u64),
            limit_count: spawn_point.limit_count,
            range: spawn_point.range,
            tactic_points: spawn_point.tactic_points,

            time_since_last_check: Duration::from_millis(0),
            current_tactics_value: 0,
            monsters: Vec::new(),
        }
    }
}
