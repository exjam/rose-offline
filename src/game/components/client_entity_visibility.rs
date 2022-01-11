use bevy_ecs::prelude::Component;

use crate::game::resources::ClientEntitySet;

#[derive(Component, Default)]
pub struct ClientEntityVisibility {
    pub entities: ClientEntitySet,
}

impl ClientEntityVisibility {
    pub fn new() -> Self {
        Default::default()
    }
}
