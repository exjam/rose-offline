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
    pub damage_sources: Vec<DamageSource>,
}

impl DamageSources {
    pub fn new() -> Self {
        Self::default()
    }
}
