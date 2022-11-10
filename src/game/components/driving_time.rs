use std::time::Duration;

use bevy::ecs::prelude::Component;

#[derive(Component)]
pub struct DrivingTime {
    pub time: Duration,
}

impl DrivingTime {
    pub fn default() -> Self {
        Self {
            time: Duration::from_secs(0),
        }
    }
}
