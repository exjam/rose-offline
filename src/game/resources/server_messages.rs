use legion::Entity;

use crate::game::{components::Position, messages::server::ServerMessage};

pub struct GlobalMessage {
    pub message: ServerMessage,
}

pub struct ZoneMessage {
    pub zone: u16,
    pub message: ServerMessage,
}

pub struct NearbyMessage {
    pub except_entity: Option<Entity>,
    pub position: Position,
    pub message: ServerMessage,
}

#[derive(Default)]
pub struct ServerMessages {
    pub pending_global_messages: Vec<GlobalMessage>,
    pub pending_zone_messages: Vec<ZoneMessage>,
    pub pending_nearby_messages: Vec<NearbyMessage>,
}

impl ServerMessages {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn send_global_message(&mut self, message: ServerMessage) {
        self.pending_global_messages.push(GlobalMessage { message });
    }

    pub fn send_zone_message(&mut self, zone: u16, message: ServerMessage) {
        self.pending_zone_messages
            .push(ZoneMessage { zone, message });
    }

    pub fn send_nearby_message(&mut self, position: Position, message: ServerMessage) {
        self.pending_nearby_messages.push(NearbyMessage {
            except_entity: None,
            position,
            message,
        });
    }

    pub fn send_nearby_except_entity_message(
        &mut self,
        except_entity: Entity,
        position: Position,
        message: ServerMessage,
    ) {
        self.pending_nearby_messages.push(NearbyMessage {
            except_entity: Some(except_entity),
            position,
            message,
        });
    }
}
