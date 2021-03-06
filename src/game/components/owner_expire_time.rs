use std::time::Instant;

use bevy::ecs::prelude::Component;

#[derive(Component)]
pub struct OwnerExpireTime {
    pub when: Instant,
}

impl OwnerExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
