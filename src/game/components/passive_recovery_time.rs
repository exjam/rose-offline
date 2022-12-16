use std::time::Duration;

use bevy::ecs::prelude::Component;

#[derive(Component)]
pub struct PassiveRecoveryTime {
    pub time: Duration,
}

impl Default for PassiveRecoveryTime {
    fn default() -> Self {
        Self {
            time: Duration::from_secs(0),
        }
    }
}
