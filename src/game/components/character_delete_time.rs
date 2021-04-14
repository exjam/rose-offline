use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

const DELETE_CHARACTER_DURATION: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Deserialize, Serialize)]
pub struct CharacterDeleteTime {
    pub start_time: SystemTime,
}

impl CharacterDeleteTime {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
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
