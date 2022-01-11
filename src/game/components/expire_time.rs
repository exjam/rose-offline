use std::time::Instant;

use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct ExpireTime {
    pub when: Instant,
}

impl ExpireTime {
    pub fn new(when: Instant) -> Self {
        Self { when }
    }
}
