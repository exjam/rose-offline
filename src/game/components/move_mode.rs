use bevy_ecs::prelude::Component;

#[derive(Component, Copy, Clone)]
pub enum MoveMode {
    Walk,
    Run,
}
