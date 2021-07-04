use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct StatPoints {
    pub points: u32,
}
