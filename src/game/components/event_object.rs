use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct EventObject {
    pub event_id: u16,
}

impl EventObject {
    pub fn new(event_id: u16) -> Self {
        Self { event_id }
    }
}
