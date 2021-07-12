use std::time::Instant;

use legion::Entity;

pub struct DamageSource {
    pub entity: Entity,
    pub total_damage: usize,
    pub first_damage_time: Instant,
    pub last_damage_time: Instant,
}

#[derive(Default)]
pub struct DamageSources {
    pub max_damage_sources: usize,
    pub damage_sources: Vec<DamageSource>,
}

impl DamageSources {
    pub fn new(max_damage_sources: usize) -> Self {
        Self {
            max_damage_sources,
            damage_sources: Vec::with_capacity(max_damage_sources),
        }
    }
}
