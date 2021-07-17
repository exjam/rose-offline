use std::time::{Duration, Instant};

pub struct ServerTime {
    pub delta: Duration,
    pub now: Instant,
}
