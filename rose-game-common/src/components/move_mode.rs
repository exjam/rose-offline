use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MoveMode {
    Walk,
    Run,
    Drive,
}
