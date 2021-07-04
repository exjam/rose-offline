use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct StatPoints {
    pub points: u32,
}

impl StatPoints {
    pub fn new() -> Self {
        Self { points: 0 }
    }
}
