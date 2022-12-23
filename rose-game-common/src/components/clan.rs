use std::num::{NonZeroU16, NonZeroU32};

use bevy::prelude::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanUniqueId(pub NonZeroU32);

impl ClanUniqueId {
    pub fn new(n: u32) -> Option<ClanUniqueId> {
        NonZeroU32::new(n).map(ClanUniqueId)
    }
}

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanLevel(pub NonZeroU32);

impl ClanLevel {
    pub fn new(n: u32) -> Option<ClanLevel> {
        NonZeroU32::new(n).map(ClanLevel)
    }
}

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanPoints(pub u64);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ClanMark {
    Premade {
        background: NonZeroU16,
        foreground: NonZeroU16,
    },
    Custom {
        crc16: u16,
    },
}
