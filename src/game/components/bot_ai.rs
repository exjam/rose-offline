use bevy_ecs::prelude::Entity;

pub enum BotAiState {
    Default,
    PickupItem(Entity),
}

pub struct BotAi {
    pub state: BotAiState,
}

impl BotAi {
    pub fn new() -> Self {
        Self {
            state: BotAiState::Default,
        }
    }
}
