use std::{
    collections::{hash_map::Iter, HashMap},
    str::FromStr,
};

use bevy::reflect::{FromReflect, Reflect};
use serde::{Deserialize, Serialize};

use crate::ZoneId;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, Reflect, FromReflect)]
pub struct WarpGateId(u16);

id_wrapper_impl!(WarpGateId, u16);

pub struct WarpGateData {
    pub target_zone: ZoneId,
    pub target_event_object: String,
}

pub struct WarpGateDatabase {
    warp_gates: HashMap<WarpGateId, WarpGateData>,
}

impl WarpGateDatabase {
    pub fn new(warp_gates: HashMap<WarpGateId, WarpGateData>) -> Self {
        Self { warp_gates }
    }

    pub fn iter(&self) -> Iter<'_, WarpGateId, WarpGateData> {
        self.warp_gates.iter()
    }

    pub fn get_warp_gate(&self, id: WarpGateId) -> Option<&WarpGateData> {
        self.warp_gates.get(&id)
    }
}
