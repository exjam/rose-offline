use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillPoints {
    pub points: u32,
}

impl SkillPoints {
    pub fn new() -> Self {
        Self { points: 0 }
    }
}
