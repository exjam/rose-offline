use std::time::{Duration, Instant};

pub struct DeltaTime {
    pub delta: Duration,
    pub now: Instant,
}
