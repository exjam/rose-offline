use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::item::Item;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActiveQuest {
    pub quest_id: usize,
    pub expire_time: Option<u32>,
    pub variables: [u16; 10],
    pub switches: BitArr!(for 32, in Lsb0, u32),
    pub items: [Option<Item>; 6],
}

impl ActiveQuest {
    pub fn new(quest_id: usize, expire_time: Option<u32>) -> Self {
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

    pub fn find_active_quest_index(&self, quest_id: usize) -> Option<usize> {
        for i in 0..self.active_quests.len() {
            if let Some(active_quest) = &self.active_quests[i] {
                if active_quest.quest_id == quest_id {
                    return Some(i);
                }
            }
        }

        None
    }

    pub fn try_add_quest(&mut self, quest: ActiveQuest) -> Option<usize> {
        for i in 0..self.active_quests.len() {
            if self.active_quests[i].is_none() {
                self.active_quests[i] = Some(quest);
                return Some(i);
            }
        }

        None
    }

    pub fn get_quest_slot_mut(&mut self, index: usize) -> Option<&mut Option<ActiveQuest>> {
        self.active_quests.get_mut(index)
    }
}
