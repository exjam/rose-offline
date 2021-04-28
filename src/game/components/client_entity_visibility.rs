use legion::Entity;
use std::collections::HashSet;

#[derive(Default)]
pub struct ClientEntityVisibility {
    pub entities: HashSet<Entity>,
}

impl ClientEntityVisibility {
    pub fn new() -> Self {
        Default::default()
    }
}
