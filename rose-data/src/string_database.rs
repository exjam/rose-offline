use arrayvec::ArrayString;
use enum_map::EnumMap;
use std::fmt::Write;

use rose_file_readers::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry};

use crate::{AbilityType, ItemClass, ItemType, SkillTargetFilter, SkillType};

// Strictly speaking we should abstract away from StlFile here, but it is not worth
// the effort until a ROSE version comes along which does not use STL...
pub struct StringDatabase {
    pub language: usize,

    pub encode_ability_type: Box<dyn Fn(AbilityType) -> Option<usize> + Send + Sync>,
    pub encode_item_class: Box<dyn Fn(ItemClass) -> Option<usize> + Send + Sync>,
    pub encode_skill_target_filter: Box<dyn Fn(SkillTargetFilter) -> Option<usize> + Send + Sync>,
    pub encode_skill_type: Box<dyn Fn(SkillType) -> Option<usize> + Send + Sync>,

    pub ability: StlFile,
    pub clan: StlFile,
    pub client_strings: StlFile,
    pub item: EnumMap<ItemType, StlFile>,
    pub item_prefix: StlFile,
    pub item_class: StlFile,
    pub job: StlFile,
    pub job_class: StlFile,
    pub npc: StlFile,
    pub npc_store_tabs: StlFile,
    pub planet: StlFile,
    pub quest: StlFile,
    pub skill: StlFile,
    pub skill_target: StlFile,
    pub skill_type: StlFile,
    pub status_effect: StlFile,
    pub union: StlFile,
    pub zone: StlFile,
}

impl StringDatabase {
    pub fn get_ability_type(&self, ability_type: AbilityType) -> &str {
        let index = if let Some(index) = (self.encode_ability_type)(ability_type) {
            index as u16
        } else {
            return "";
        };

        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", index).ok();
        self.ability
            .get_text_string(self.language, &key)
            .unwrap_or("")
    }

    pub fn get_item(&self, item_type: ItemType, key: &str) -> Option<StlItemEntry> {
        let index = self.item[item_type].lookup_key(key)?;
        self.item[item_type].get_item_entry(self.language, index)
    }

    pub fn get_item_class(&self, item_class: ItemClass) -> &str {
        let index = if let Some(index) = (self.encode_item_class)(item_class) {
            index as u16
        } else {
            return "";
        };
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", index).ok();
        self.item_class
            .get_text_string(self.language, &key)
            .unwrap_or("")
    }

    pub fn get_job_name(&self, job: u16) -> &str {
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", job).ok();
        self.job.get_text_string(self.language, &key).unwrap_or("")
    }

    pub fn get_job_class_name(&self, key: &str) -> &str {
        self.job_class
            .get_text_string(self.language, key)
            .unwrap_or("")
    }

    pub fn get_npc(&self, key: &str) -> Option<StlNormalEntry> {
        let index = self.npc.lookup_key(key)?;
        self.npc.get_normal_entry(self.language, index)
    }

    pub fn get_npc_store_tab(&self, key: &str) -> Option<StlNormalEntry> {
        let index = self.npc_store_tabs.lookup_key(key)?;
        self.npc_store_tabs.get_normal_entry(self.language, index)
    }

    pub fn get_quest(&self, key: &str) -> Option<StlQuestEntry> {
        let index = self.quest.lookup_key(key)?;
        self.quest.get_quest_entry(self.language, index)
    }

    pub fn get_skill(&self, key: &str) -> Option<StlItemEntry> {
        let index = self.skill.lookup_key(key)?;
        self.skill.get_item_entry(self.language, index)
    }

    pub fn get_skill_target_filter(&self, skill_target_filter: SkillTargetFilter) -> &str {
        let index = if let Some(index) = (self.encode_skill_target_filter)(skill_target_filter) {
            index as u16
        } else {
            return "";
        };
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", index).ok();
        self.skill_target
            .get_text_string(self.language, &key)
            .unwrap_or("")
    }

    pub fn get_skill_type(&self, skill_type: SkillType) -> &str {
        let index = if let Some(index) = (self.encode_skill_type)(skill_type) {
            index as u16
        } else {
            return "";
        };
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", index).ok();
        self.skill_type
            .get_text_string(self.language, &key)
            .unwrap_or("")
    }

    pub fn get_status_effect(&self, key: &str) -> Option<StlQuestEntry> {
        let index = self.status_effect.lookup_key(key)?;
        self.status_effect.get_quest_entry(self.language, index)
    }

    pub fn get_zone(&self, key: &str) -> Option<StlItemEntry> {
        let index = self.zone.lookup_key(key)?;
        self.zone.get_item_entry(self.language, index)
    }
}
