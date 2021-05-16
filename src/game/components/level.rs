use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Level {
    pub level: u16,
    pub xp: u64,
}

impl Default for Level {
    fn default() -> Self {
        Self { level: 1, xp: 0 }
    }
}

impl Level {
    pub fn new(level: u16) -> Self {
        Self { level, xp: 0 }
    }
}
