use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HotbarSlot {
    Inventory(u16),
    Command(u16),
    Skill(u16),
    Emote(u16),
    Dialog(u16),
    ClanSkill(u16),
}

const HOTBAR_PAGE_SIZE: usize = 8;
const HOTBAR_NUM_PAGES: usize = 4;

#[derive(Component, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Hotbar {
    pub pages: [[Option<HotbarSlot>; HOTBAR_PAGE_SIZE]; HOTBAR_NUM_PAGES],
}

impl Hotbar {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_slot(&mut self, index: usize, slot: Option<HotbarSlot>) -> Option<()> {
        let page = self.pages.get_mut(index / HOTBAR_PAGE_SIZE)?;
        let page_slot = page.get_mut(index % HOTBAR_PAGE_SIZE)?;
        *page_slot = slot;
        Some(())
    }
}
