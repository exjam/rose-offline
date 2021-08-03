use bevy_ecs::prelude::Entity;
use nalgebra::Point3;

#[derive(Clone, Copy)]
pub enum SpawnOrigin {
    MonsterSpawnPoint(Entity, Point3<f32>),
}
