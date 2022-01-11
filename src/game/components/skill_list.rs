use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::data::{SkillData, SkillId, SkillPageType};

const SKILL_PAGE_SIZE: usize = 30;

#[derive(Copy, Clone, Debug)]
pub struct SkillSlot(pub SkillPageType, pub usize);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillPage {
    pub page_type: SkillPageType,
    pub skills: [Option<SkillId>; SKILL_PAGE_SIZE],
}

impl SkillPage {
    pub fn new(page_type: SkillPageType) -> Self {
        Self {
            page_type,
            skills: Default::default(),
        }
    }

    pub fn add_skill(&mut self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        let (index, empty_slot) = self
            .skills
            .iter_mut()
            .enumerate()
            .find(|(_, x)| x.is_none())?;
        *empty_slot = Some(skill_data.id);
        Some((SkillSlot(self.page_type, index), skill_data.id))
    }

    pub fn find_skill(&self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.skills
            .iter()
            .enumerate()
            .find(|(_, slot)| **slot == Some(skill_data.id))
            .map(|(index, _)| (SkillSlot(self.page_type, index), skill_data.id))
    }
}

#[derive(Component, Clone, Debug, Deserialize, Serialize)]
pub struct SkillList {
    pub basic: SkillPage,
    pub active: SkillPage,
    pub passive: SkillPage,
    pub clan: SkillPage,
}

impl SkillList {
    pub fn new() -> Self {
        Self {
            basic: SkillPage::new(SkillPageType::Basic),
            active: SkillPage::new(SkillPageType::Active),
            passive: SkillPage::new(SkillPageType::Passive),
            clan: SkillPage::new(SkillPageType::Clan),
        }
    }

    #[allow(dead_code)]
    fn get_page(&self, page_type: SkillPageType) -> &SkillPage {
        match page_type {
            SkillPageType::Basic => &self.basic,
            SkillPageType::Active => &self.active,
            SkillPageType::Passive => &self.passive,
            SkillPageType::Clan => &self.clan,
        }
    }

    fn get_page_mut(&mut self, page_type: SkillPageType) -> &mut SkillPage {
        match page_type {
            SkillPageType::Basic => &mut self.basic,
            SkillPageType::Active => &mut self.active,
            SkillPageType::Passive => &mut self.passive,
            SkillPageType::Clan => &mut self.clan,
        }
    }

    pub fn add_skill(&mut self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.get_page_mut(skill_data.page).add_skill(skill_data)
    }

    pub fn find_skill(&mut self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.get_page_mut(skill_data.page).find_skill(skill_data)
    }

    pub fn get_skill(&self, skill_slot: SkillSlot) -> Option<SkillId> {
        self.get_page(skill_slot.0)
            .skills
            .get(skill_slot.1)
            .copied()
            .flatten()
    }

    pub fn get_passive_skills(&self) -> impl Iterator<Item = &SkillId> + '_ {
        self.passive.skills.iter().filter_map(|x| x.as_ref())
    }

    pub fn get_slot_mut(&mut self, skill_slot: SkillSlot) -> Option<&mut Option<SkillId>> {
        self.get_page_mut(skill_slot.0).skills.get_mut(skill_slot.1)
    }

    pub fn iter_skills(&self) -> impl Iterator<Item = &SkillId> {
        self.basic
            .skills
            .iter()
            .chain(self.active.skills.iter())
            .chain(self.passive.skills.iter())
            .chain(self.clan.skills.iter())
            .filter_map(|x| x.as_ref())
    }
}
