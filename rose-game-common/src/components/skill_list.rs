use bevy::ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use rose_data::{SkillData, SkillDatabase, SkillId, SkillPageType};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct SkillSlot(pub SkillPageType, pub usize);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillPage {
    pub page_type: SkillPageType,
    pub skills: Vec<Option<SkillId>>,
}

impl SkillPage {
    pub fn new(page_type: SkillPageType, size: usize) -> Self {
        Self {
            page_type,
            skills: vec![None; size],
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

    pub fn remove_skill(&mut self, skill_data: &SkillData) -> Option<SkillSlot> {
        let (skill_slot, _) = self.find_skill_exact(skill_data)?;
        self.skills[skill_slot.1].take();
        Some(skill_slot)
    }

    pub fn find_skill_exact(&self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.skills
            .iter()
            .enumerate()
            .find(|(_, slot)| **slot == Some(skill_data.id))
            .map(|(index, _)| (SkillSlot(self.page_type, index), skill_data.id))
    }

    pub fn find_skill_level(
        &self,
        skill_database: &SkillDatabase,
        base_skill_id: SkillId,
    ) -> Option<(SkillSlot, SkillId, u32)> {
        for (index, slot) in self.skills.iter().enumerate() {
            if let Some(skill_data) = slot.and_then(|id| skill_database.get_skill(id)) {
                if skill_data.base_skill_id == Some(base_skill_id) {
                    return Some((
                        SkillSlot(self.page_type, index),
                        skill_data.id,
                        skill_data.level,
                    ));
                }
            }
        }

        None
    }
}

#[derive(Component, Clone, Debug, Default, Deserialize, Serialize)]
pub struct SkillList {
    pub pages: Vec<SkillPage>,
}

impl SkillList {
    pub fn get_page(&self, page_type: SkillPageType) -> Option<&SkillPage> {
        self.pages.iter().find(|&page| page.page_type == page_type)
    }

    pub fn get_page_mut(&mut self, page_type: SkillPageType) -> Option<&mut SkillPage> {
        self.pages
            .iter_mut()
            .find(|page| page.page_type == page_type)
    }

    pub fn add_skill(&mut self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.get_page_mut(skill_data.page)
            .and_then(|page| page.add_skill(skill_data))
    }

    pub fn remove_skill(&mut self, skill_data: &SkillData) -> Option<SkillSlot> {
        self.get_page_mut(skill_data.page)
            .and_then(|page| page.remove_skill(skill_data))
    }

    pub fn find_skill_exact(&self, skill_data: &SkillData) -> Option<(SkillSlot, SkillId)> {
        self.get_page(skill_data.page)
            .and_then(|page| page.find_skill_exact(skill_data))
    }

    pub fn find_skill_level(
        &self,
        skill_database: &SkillDatabase,
        base_skill_id: SkillId,
    ) -> Option<(SkillSlot, SkillId, u32)> {
        if let Some(skill_data) = skill_database.get_skill(base_skill_id) {
            self.get_page(skill_data.page)
                .and_then(|page| page.find_skill_level(skill_database, base_skill_id))
        } else {
            None
        }
    }

    pub fn get_skill(&self, skill_slot: SkillSlot) -> Option<SkillId> {
        self.get_page(skill_slot.0)
            .and_then(|page| page.skills.get(skill_slot.1))
            .copied()
            .flatten()
    }

    pub fn get_slot_mut(&mut self, skill_slot: SkillSlot) -> Option<&mut Option<SkillId>> {
        self.get_page_mut(skill_slot.0)
            .and_then(|page| page.skills.get_mut(skill_slot.1))
    }
}
