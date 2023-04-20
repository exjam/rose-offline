use std::time::{Duration, SystemTime};

use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

const DELETE_CHARACTER_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Component, Copy, Clone, Debug, Deserialize, Serialize)]
pub struct CharacterDeleteTime {
    pub start_time: SystemTime,
}

impl CharacterDeleteTime {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
        }
    }

    pub fn from_seconds_remaining(seconds: u32) -> Self {
        Self {
            start_time: SystemTime::now()
                - (DELETE_CHARACTER_DURATION - Duration::new(seconds as u64, 0)),
        }
    }

    pub fn get_time_until_delete(&self) -> Duration {
        let time_since_delete = self.start_time.elapsed().unwrap();

        if time_since_delete < DELETE_CHARACTER_DURATION {
            DELETE_CHARACTER_DURATION - time_since_delete
        } else {
            Duration::new(0, 0)
        }
    }
}

impl Default for CharacterDeleteTime {
    fn default() -> Self {
        Self::new()
    }
}
