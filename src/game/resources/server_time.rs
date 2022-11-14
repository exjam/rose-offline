use bevy::prelude::Resource;
use chrono::prelude::{DateTime, Local};
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct ServerTime {
    pub delta: Duration,
    pub now: Instant,
    pub local_time: DateTime<Local>,
}
