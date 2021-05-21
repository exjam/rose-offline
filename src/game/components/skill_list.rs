use serde::{Deserialize, Serialize};

use crate::data::{SkillPage, SkillReference};

const SKILL_PAGE_SIZE: usize = 30;
const SKILL_NUM_PAGES: usize = 4;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillList {
    pub pages: [[Option<SkillReference>; SKILL_PAGE_SIZE]; SKILL_NUM_PAGES],
}

fn get_page_index(page: SkillPage) -> usize {
    match page {
        SkillPage::Basic => 0,
        SkillPage::Active => 1,
        SkillPage::Passive => 2,
        SkillPage::Clan => 3,
    }
}

impl Default for SkillList {
    fn default() -> Self {
        Self {
            pages: Default::default(),
        }
    }
}

impl SkillList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_skill(&mut self, skill: SkillReference, page: SkillPage) -> Option<u16> {
        let page_index = get_page_index(page);
        let page = self.pages.get_mut(page_index)?;
        let (slot_index, empty_slot) = page.iter_mut().enumerate().find(|(_, x)| x.is_none())?;
        *empty_slot = Some(skill);
        Some((page_index * SKILL_PAGE_SIZE + slot_index) as u16)
    }

    pub fn get_passive_skills(&self) -> impl Iterator<Item = &SkillReference> + '_ {
        self.pages[1].iter().filter_map(|x| x.as_ref())
    }
}
