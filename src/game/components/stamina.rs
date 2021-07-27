use serde::{Deserialize, Serialize};

pub const MAX_STAMINA: u32 = 5000;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Stamina {
    pub stamina: u32,
}

impl Stamina {
    pub fn new() -> Self {
        Self { stamina: 0 }
    }
}
