use std::num::NonZeroUsize;

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct UnionMembership {
    pub current_union: Option<NonZeroUsize>,
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
