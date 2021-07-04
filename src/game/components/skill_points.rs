use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct SkillPoints {
    pub points: u32,
}
