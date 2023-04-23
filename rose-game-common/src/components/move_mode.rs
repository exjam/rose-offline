use bevy::{
    ecs::prelude::Component,
    reflect::{FromReflect, Reflect},
};
use serde::{Deserialize, Serialize};

#[derive(
    Component, Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect, FromReflect,
)]
pub enum MoveMode {
    Walk,
    Run,
    Drive,
}
