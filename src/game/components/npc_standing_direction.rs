use bevy::ecs::prelude::Component;

#[derive(Component, Clone)]
pub struct NpcStandingDirection {
    pub direction: f32,
}

impl NpcStandingDirection {
    pub fn new(direction: f32) -> Self {
        Self { direction }
    }
}
