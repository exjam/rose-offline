use crate::game::data::STB_SKILL;
use serde::{Deserialize, Serialize};

const SKILL_PAGE_SIZE: usize = 30;
const SKILL_NUM_PAGES: usize = 4;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkillList {
    pub pages: [[Option<u16>; SKILL_PAGE_SIZE]; SKILL_NUM_PAGES],
}

impl Default for SkillList {
    fn default() -> Self {
        let mut skill_list = Self {
            pages: Default::default(),
        };
        skill_list.learn_skill(11);
        skill_list.learn_skill(12);
        skill_list.learn_skill(16);
        skill_list.learn_skill(19);
        skill_list.learn_skill(20);
        skill_list.learn_skill(21);
        skill_list
    }
}

impl SkillList {
    pub fn learn_skill(&mut self, id: u16) -> Option<u16> {
        let page_index = STB_SKILL.get_skill_tab_type(id as usize)? as usize;
        let page = self.pages.get_mut(page_index)?;
        let (slot_index, empty_slot) = page.iter_mut().enumerate().find(|(_, x)| x.is_none())?;
        *empty_slot = Some(id);
        Some((page_index * SKILL_PAGE_SIZE + slot_index) as u16)
    }
}
