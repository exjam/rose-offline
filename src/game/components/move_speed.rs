use bevy_ecs::prelude::Component;

#[derive(Component, Copy, Clone)]
pub struct MoveSpeed {
    pub speed: f32,
}

impl MoveSpeed {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}
