use num_derive::NumOps;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};

pub const WORLD_TICK_DURATION: Duration = Duration::from_secs(10);

pub const WORLD_MONTH_PER_YEAR: u64 = 12;
pub const WORLD_DAYS_PER_MONTH: u64 = 54;

pub const WORLD_TICKS_PER_MONTH: u64 = 8640;
pub const WORLD_TICKS_PER_YEAR: u64 = 103680;
pub const WORLD_TICKS_PER_DAY: u64 = WORLD_TICKS_PER_MONTH / WORLD_DAYS_PER_MONTH;

#[derive(Copy, Clone, Debug, NumOps, Deserialize, Serialize)]
pub struct WorldTicks(pub u64);

impl From<WorldTicks> for Duration {
    fn from(ticks: WorldTicks) -> Duration {
        Duration::from_millis(ticks.0 * WORLD_TICK_DURATION.as_millis() as u64)
    }
}

impl FromStr for WorldTicks {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u64>().map_err(|_| ())?;
        Ok(WorldTicks(value))
    }
}

impl WorldTicks {
    pub fn get_world_time(&self) -> u32 {
        (self.0 % WORLD_TICKS_PER_MONTH) as u32
    }

    pub fn get_world_day(&self) -> u32 {
        (self.get_world_time() / WORLD_TICKS_PER_DAY as u32) + 1
    }

    pub fn get_world_month(&self) -> u32 {
        (self.0 / WORLD_TICKS_PER_MONTH) as u32
    }

    pub fn get_world_year(&self) -> u32 {
        (self.0 / WORLD_TICKS_PER_YEAR) as u32
    }
}
