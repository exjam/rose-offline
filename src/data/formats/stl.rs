use std::{
    collections::{hash_map::Keys, HashMap},
    str,
};

use super::{reader::ReadError, FileReader};

struct StlEntry {
    pub text_offset: u32,
    pub comment_offset: u32,
    pub quest1_offset: u32,
    pub quest2_offset: u32,
}

#[derive(Default)]
struct StlLanguage {
    text: Vec<(u32, u32)>,
    comment: Vec<(u32, u32)>,
    quest1: Vec<(u32, u32)>,
    quest2: Vec<(u32, u32)>,
}

pub struct StlFile {
    data: Vec<u8>,
    string_keys: HashMap<String, u32>,
    integer_keys: HashMap<u32, u32>,
    languages: Vec<StlLanguage>,
}

pub struct StlNormalEntry<'a> {
    pub text: &'a str,
}

pub struct StlItemEntry<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

pub struct StlQuestEntry<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub start_message: &'a str,
    pub end_message: &'a str,
}

#[derive(Debug)]
pub enum StlReadError {
    InvalidType,
    UnexpectedEof,
}

impl From<ReadError> for StlReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => StlReadError::UnexpectedEof,
        }
    }
}

enum StlType {
    Item,
    Normal,
    Quest,
}

impl StlFile {
    pub fn read(mut reader: FileReader) -> Result<Self, StlReadError> {
        let stl_type_str = reader.read_u8_length_string()?;
        let stl_type = if stl_type_str == "ITST01" {
            StlType::Item
        } else if stl_type_str == "NRST01" {
            StlType::Normal
        } else if stl_type_str == "QEST01" {
            StlType::Quest
        } else {
            return Err(StlReadError::InvalidType);
        };

        let key_count = reader.read_u32()?;
        let mut string_keys = HashMap::new();
        let mut integer_keys = HashMap::new();
        for i in 0..key_count {
            let key = reader.read_u8_length_string()?;
            let index = reader.read_u32()?;
            string_keys.insert(key.to_string(), i);
            integer_keys.insert(index, i);
        }

        let language_count = reader.read_u32()?;
        let mut data = Vec::new();

        let read_stl_entry =
            |reader: &mut FileReader, data: &mut Vec<u8>| -> Result<(u32, u32), StlReadError> {
                let text = reader.read_u8_length_string()?;
                let text_bytes = text.as_bytes();
                let text_bytes_length = text_bytes.len();
                let text_offset = data.len();
                data.extend_from_slice(text_bytes);
                Ok((text_offset as u32, text_bytes_length as u32))
            };

        let mut languages = Vec::new();
        for _ in 0..language_count {
            let language_offset = reader.read_u32()?;
            let language_save_position = reader.position();
            let mut language = StlLanguage::default();
            reader.set_position(language_offset as u64);

            for _ in 0..key_count {
                let entry_offset = reader.read_u32()?;
                let entry_save_position = reader.position();
                reader.set_position(entry_offset as u64);

                match stl_type {
                    StlType::Normal => {
                        language.text.push(read_stl_entry(&mut reader, &mut data)?);
                    }
                    StlType::Item => {
                        language.text.push(read_stl_entry(&mut reader, &mut data)?);
                        language
                            .comment
                            .push(read_stl_entry(&mut reader, &mut data)?);
                    }
                    StlType::Quest => {
                        language.text.push(read_stl_entry(&mut reader, &mut data)?);
                        language
                            .comment
                            .push(read_stl_entry(&mut reader, &mut data)?);
                        language
                            .quest1
                            .push(read_stl_entry(&mut reader, &mut data)?);
                        language
                            .quest2
                            .push(read_stl_entry(&mut reader, &mut data)?);
                    }
                }

                reader.set_position(entry_save_position);
            }

            languages.push(language);
            reader.set_position(language_save_position);
        }

        Ok(StlFile {
            data,
            string_keys,
            integer_keys,
            languages,
        })
    }

    pub fn keys(&self) -> Keys<'_, String, u32> {
        self.string_keys.keys()
    }

    pub fn get_text_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language as usize)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.text.get(*index as usize)?;
        str::from_utf8(&self.data[*offset as usize..(offset + size) as usize]).ok()
    }

    pub fn get_comment_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language as usize)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.comment.get(*index as usize)?;
        str::from_utf8(&self.data[*offset as usize..(offset + size) as usize]).ok()
    }

    pub fn get_quest1_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language as usize)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.quest1.get(*index as usize)?;
        str::from_utf8(&self.data[*offset as usize..(offset + size) as usize]).ok()
    }

    pub fn get_quest2_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language as usize)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.quest2.get(*index as usize)?;
        str::from_utf8(&self.data[*offset as usize..(offset + size) as usize]).ok()
    }

    pub fn get_normal_entry(&self, language: usize, key: &str) -> Option<StlNormalEntry<'_>> {
        Some(StlNormalEntry {
            text: self.get_text_string(language, key)?,
        })
    }

    pub fn get_item_entry(&self, language: usize, key: &str) -> Option<StlItemEntry<'_>> {
        Some(StlItemEntry {
            name: self.get_text_string(language, key)?,
            description: self.get_comment_string(language, key)?,
        })
    }

    pub fn get_quest_entry(&self, language: usize, key: &str) -> Option<StlQuestEntry<'_>> {
        Some(StlQuestEntry {
            name: self.get_text_string(language, key)?,
            description: self.get_comment_string(language, key)?,
            start_message: self.get_quest1_string(language, key)?,
            end_message: self.get_quest2_string(language, key)?,
        })
    }
}
