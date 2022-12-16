use bevy::prelude::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanUniqueId(pub u32);

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanLevel(pub u32);

#[derive(Deref, DerefMut, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ClanPoints(pub u64);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ClanMemberPosition {
    Penalty,
    Junior,
    Senior,
    Veteran,
    Commander,
    DeputyMaster,
    Master,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ClanMark {
    Premade { foreground: u16, background: u16 },
    Custom { crc16: u16 },
}
