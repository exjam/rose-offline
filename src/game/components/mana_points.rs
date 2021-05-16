use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct ManaPoints {
    pub mp: u32,
}

impl ManaPoints {
    pub fn new(mp: u32) -> Self {
        Self { mp }
    }
}
