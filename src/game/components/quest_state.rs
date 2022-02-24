use bevy_ecs::prelude::Component;
use bitvec::prelude::*;
use serde::{Deserialize, Serialize};

use crate::data::{
    item::{Item, ItemSlotBehaviour},
    ItemReference, WorldTicks,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActiveQuest {
    pub quest_id: usize,
    pub expire_time: Option<WorldTicks>,
    pub variables: [u16; 10],
    pub switches: BitArr!(for 32, in u32, Lsb0),
    pub items: [Option<Item>; 6],
}

impl ActiveQuest {
    pub fn new(quest_id: usize, expire_time: Option<WorldTicks>) -> Self {
        Self {
            quest_id,
            expire_time,
            variables: Default::default(),
            switches: Default::default(),
            items: Default::default(),
        }
    }

    pub fn find_item(&self, item_reference: ItemReference) -> Option<&Item> {
        for item in self.items.iter() {
            if let Some(item) = item.as_ref() {
                if item.is_same_item_reference(item_reference) {
                    return Some(item);
                }
            }
        }

        None
    }

    pub fn try_add_item(&mut self, item: Item) -> Result<usize, Item> {
        // First try stack with any other existing items
        for i in 0..self.items.len() {
            if let Some(quest_item) = &mut self.items[i] {
                if quest_item.try_stack_with_item(item.clone()).is_ok() {
                    return Ok(i);
                }
            }
        }

        // Else find empty slot
        for i in 0..self.items.len() {
            if self.items[i].is_none() {
                self.items[i] = Some(item);
                return Ok(i);
            }
        }

        Err(item)
    }

    pub fn try_take_item(&mut self, item_reference: ItemReference, quantity: u32) -> Option<Item> {
        for i in 0..self.items.len() {
            if let Some(quest_item) = &mut self.items[i] {
                if quest_item.is_same_item_reference(item_reference) {
                    if let Some(taken_item) = self.items[i].try_take_quantity(quantity) {
                        return Some(taken_item);
                    }
                }
            }
        }

        None
    }
}

#[derive(Component, Clone, Debug, Default, Deserialize, Serialize)]
pub struct QuestState {
    pub episode_variables: [u16; 5],
    pub job_variables: [u16; 3],
    pub planet_variables: [u16; 7],
    pub union_variables: [u16; 10],
    pub quest_switches: BitArr!(for 1024, in u32, Lsb0),
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

    pub fn get_quest(&self, index: usize) -> Option<&ActiveQuest> {
        self.active_quests.get(index).and_then(|x| x.as_ref())
    }

    pub fn get_quest_mut(&mut self, index: usize) -> Option<&mut ActiveQuest> {
        self.active_quests.get_mut(index).and_then(|x| x.as_mut())
    }

    pub fn get_quest_slot_mut(&mut self, index: usize) -> Option<&mut Option<ActiveQuest>> {
        self.active_quests.get_mut(index)
    }
}
