use std::collections::HashMap;

use crate::data::formats::AipFile;

pub struct AiDatabase {
    pub strings: HashMap<u16, String>,
    pub aips: HashMap<u16, AipFile>,
}

impl AiDatabase {
    pub fn get_ai(&self, index: usize) -> Option<&AipFile> {
        self.aips.get(&(index as u16))
    }

    pub fn get_ai_string(&self, index: usize) -> Option<&str> {
        self.strings.get(&(index as u16)).map(String::as_str)
    }
}
