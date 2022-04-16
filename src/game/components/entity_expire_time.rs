use std::time::Instant;

use bevy::ecs::prelude::Component;

#[derive(Component)]
pub struct EntityExpireTime {
    pub when: Instant,
}

impl EntityExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
