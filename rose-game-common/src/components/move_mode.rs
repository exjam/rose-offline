use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveMode {
    Walk,
    Run,
    Drive,
}
