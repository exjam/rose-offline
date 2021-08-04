use chrono::prelude::{DateTime, Local};
use std::time::{Duration, Instant};

pub struct ServerTime {
    pub delta: Duration,
    pub now: Instant,
    pub local_time: DateTime<Local>,
}
