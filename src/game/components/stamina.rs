use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Stamina {
    pub stamina: u32,
}

impl Stamina {
    pub fn new() -> Self {
        Self { stamina: 0 }
    }
}
