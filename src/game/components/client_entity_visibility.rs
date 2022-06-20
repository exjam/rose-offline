use bevy::prelude::{Component, Deref, DerefMut};

use crate::game::resources::ClientEntitySet;

#[derive(Component, Default, Deref, DerefMut)]
pub struct ClientEntityVisibility {
    pub entities: ClientEntitySet,
}

impl ClientEntityVisibility {
    pub fn new() -> Self {
        Default::default()
    }
}
