use legion::Entity;
use nalgebra::Point3;

pub enum SpawnOrigin {
    MonsterSpawnPoint(Entity, Point3<f32>),
}
