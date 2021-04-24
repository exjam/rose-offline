use crate::game::data::formats::ifo;

pub struct MonsterSpawn {
    pub id: u32,
    pub count: u32,
}

pub struct MonsterSpawnPoint {
    pub basic_spawns: Vec<MonsterSpawn>,
    pub tactic_spawns: Vec<MonsterSpawn>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
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
            interval: spawn_point.interval,
            limit_count: spawn_point.limit_count,
            range: spawn_point.range,
            tactic_points: spawn_point.tactic_points,
        }
    }
}
