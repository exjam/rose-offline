use bevy::{ecs::prelude::Component, reflect::Reflect};
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub enum MoveMode {
    Walk,
    Run,
    Drive,
}
