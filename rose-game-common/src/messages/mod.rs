use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientEntityId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyRejectInviteReason {
    Busy,
    Reject,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyXpSharing {
    EqualShare,
    DistributedByLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyItemSharing {
    EqualLootDistribution,
    AcquisitionOrder,
}
