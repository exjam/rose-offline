use bevy_ecs::prelude::Entity;

pub type PendingChatCommandList = Vec<(Entity, String)>;
