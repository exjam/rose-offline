use serde::{Deserialize, Serialize};

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
