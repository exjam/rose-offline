use std::time::Duration;

use bevy_ecs::prelude::{Component, Entity};
use rand::Rng;

pub const BOT_IDLE_CHECK_DURATION: Duration = Duration::from_secs(3);

pub enum BotAiState {
    Farm,
    PickupItem(Entity),
    SnowballFight,
}

#[derive(Component)]
pub struct BotAi {
    pub state: BotAiState,
    pub time_since_last_idle_check: Duration,
}

impl BotAi {
    pub fn new(state: BotAiState) -> Self {
        Self {
            state,
            time_since_last_idle_check: Duration::from_millis(
                rand::thread_rng().gen_range(0..=(BOT_IDLE_CHECK_DURATION.as_millis() as u64)),
            ),
        }
    }
}
