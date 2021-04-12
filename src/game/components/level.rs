use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Level {
    pub level: u16,
    pub xp: u64,
}

impl Level {
    pub fn default() -> Self {
        Self { level: 1, xp: 0 }
    }
}
