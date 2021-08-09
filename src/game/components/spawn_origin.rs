use bevy_ecs::prelude::Entity;
use nalgebra::Point3;

#[derive(Clone, Copy)]
pub enum SpawnOrigin {
    Summoned(Entity, Point3<f32>),
    MonsterSpawnPoint(Entity, Point3<f32>),
    Quest(Entity, Point3<f32>),
}
