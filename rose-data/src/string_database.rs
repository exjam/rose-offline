use arrayvec::ArrayString;
use enum_map::EnumMap;
use std::fmt::Write;

use rose_file_readers::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry};

use crate::ItemType;

// Strictly speaking we should abstract away from StlFile here, but it is not worth
// the effort until a ROSE version comes along which does not use STL...
pub struct StringDatabase {
    pub language: usize,

    pub ability: StlFile,
    pub clan: StlFile,
    pub item: EnumMap<ItemType, StlFile>,
    pub item_prefix: StlFile,
    pub item_type: StlFile,
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
    pub fn get_item(&self, item_type: ItemType, key: &str) -> Option<StlItemEntry> {
        let index = self.item[item_type].lookup_key(key)?;
        self.item[item_type].get_item_entry(self.language, index)
    }

    pub fn get_job_name(&self, job: u16) -> Option<&str> {
        let mut key = ArrayString::<16>::new();
        write!(&mut key, "{}", job).ok()?;
        self.job.get_text_string(self.language, &key)
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

    pub fn get_status_effect(&self, key: &str) -> Option<StlQuestEntry> {
        let index = self.status_effect.lookup_key(key)?;
        self.status_effect.get_quest_entry(self.language, index)
    }

    pub fn get_zone(&self, key: &str) -> Option<StlItemEntry> {
        let index = self.zone.lookup_key(key)?;
        self.zone.get_item_entry(self.language, index)
    }
}
