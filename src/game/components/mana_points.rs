use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ManaPoints {
    pub mp: u32,
}

impl ManaPoints {
    pub fn new(mp: u32) -> Self {
        Self { mp }
    }
}
