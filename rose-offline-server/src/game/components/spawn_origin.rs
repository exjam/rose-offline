use bevy::ecs::prelude::{Component, Entity};
use bevy::math::Vec3;

#[derive(Component, Clone, Copy)]
pub enum SpawnOrigin {
    Summoned(Entity, Vec3),
    MonsterSpawnPoint(Entity, Vec3),
    Quest(Entity, Vec3),
}
