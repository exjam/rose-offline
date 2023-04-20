use std::time::Duration;

use bevy::ecs::prelude::{Component, Entity};
use rand::Rng;

pub const BOT_IDLE_CHECK_DURATION: Duration = Duration::from_secs(1);

pub enum BotAiState {
    Farm,
    PickupItem(Entity),
}

pub enum BotMessage {
    PartyInvite(Entity),
}

#[derive(Component)]
pub struct BotAi {
    pub state: BotAiState,
    pub time_since_last_idle_check: Duration,
    pub messages: Vec<BotMessage>,
}

impl BotAi {
    pub fn new(state: BotAiState) -> Self {
        Self {
            state,
            time_since_last_idle_check: Duration::from_millis(
                rand::thread_rng().gen_range(0..=(BOT_IDLE_CHECK_DURATION.as_millis() as u64)),
            ),
            messages: Vec::with_capacity(8),
        }
    }
}
