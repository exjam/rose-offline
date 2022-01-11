use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct Weight {
    pub weight: u32,
}

impl Weight {
    pub fn new(weight: u32) -> Self {
        Self { weight }
    }
}
