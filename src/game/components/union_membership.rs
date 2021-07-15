use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UnionMembership {
    pub current_union: Option<usize>,
    pub points: [u32; 10],
}

impl UnionMembership {
    pub fn new() -> Self {
        Self {
            current_union: None,
            points: [0; 10],
        }
    }
}
