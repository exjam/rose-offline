use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClientEntityId(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum PartyRejectInviteReason {
    Busy,
    Reject,
}
