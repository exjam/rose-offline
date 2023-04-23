use std::num::NonZeroUsize;

use bevy::{
    ecs::prelude::Component,
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Component, Clone, Debug, Deserialize, Serialize, Reflect, FromReflect)]
pub struct UnionMembership {
    pub current_union: Option<NonZeroUsize>,
    pub points: [u32; 10],
}

impl UnionMembership {
    pub fn new() -> Self {
        Default::default()
    }
}
