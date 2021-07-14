use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::item::Item;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActiveQuest {
    pub quest_id: u16,
    pub expire_time: Option<u32>,
    pub variables: [u16; 10],
    pub switches: BitArr!(for 32, in Lsb0, u32),
    pub items: [Option<Item>; 6],
}

impl ActiveQuest {
    pub fn new(quest_id: u16, expire_time: Option<u32>) -> Self {
        Self {
            quest_id,
            expire_time,
            variables: Default::default(),
            switches: Default::default(),
            items: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuestState {
    pub episode_variables: [u16; 5],
    pub job_variables: [u16; 3],
    pub planet_variables: [u16; 7],
    pub union_variables: [u16; 10],
    pub quest_switches: BitArr!(for 1024, in Lsb0, u32),
    pub active_quests: [Option<ActiveQuest>; 10],
}

impl QuestState {
    pub fn new() -> Self {
        Default::default()
    }
}
