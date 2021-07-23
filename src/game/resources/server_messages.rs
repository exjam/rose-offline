use crate::game::{
    components::{ClientEntity, ClientEntityId},
    messages::server::ServerMessage,
};

pub struct GlobalMessage {
    pub message: ServerMessage,
}

pub struct ZoneMessage {
    pub zone: u16,
    pub message: ServerMessage,
}

pub struct EntityMessage {
    pub zone: u16,
    pub entity_id: ClientEntityId,
    pub message: ServerMessage,
}

#[derive(Default)]
pub struct ServerMessages {
    pub pending_global_messages: Vec<GlobalMessage>,
    pub pending_zone_messages: Vec<ZoneMessage>,
    pub pending_entity_messages: Vec<EntityMessage>,
}

#[allow(dead_code)]
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

    pub fn send_entity_message(&mut self, entity: &ClientEntity, message: ServerMessage) {
        self.pending_entity_messages.push(EntityMessage {
            zone: entity.zone,
            entity_id: entity.id,
            message,
        });
    }
}
