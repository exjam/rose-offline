use anyhow::anyhow;
use std::{
    collections::{hash_map::Keys, HashMap},
    str,
};

use crate::{reader::RoseFileReader, RoseFile};

struct StlLanguage {
    text: Vec<(u32, u32)>,
    comment: Vec<(u32, u32)>,
    quest1: Vec<(u32, u32)>,
    quest2: Vec<(u32, u32)>,
}

impl StlLanguage {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            text: Vec::with_capacity(capacity),
            comment: Vec::with_capacity(capacity),
            quest1: Vec::with_capacity(capacity),
            quest2: Vec::with_capacity(capacity),
        }
    }
}

#[allow(dead_code)]
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

#[derive(Default)]
pub struct StlReadOptions {
    pub language_filter: Option<Vec<usize>>,
}

enum StlType {
    Item,
    Normal,
    Quest,
}

impl RoseFile for StlFile {
    type ReadOptions = StlReadOptions;
    type WriteOptions = ();

    fn read(
        mut reader: RoseFileReader,
        read_options: &Self::ReadOptions,
    ) -> Result<Self, anyhow::Error> {
        let stl_type_str = reader.read_variable_length_string()?;
        let stl_type = if stl_type_str == "ITST01" {
            StlType::Item
        } else if stl_type_str == "NRST01" {
            StlType::Normal
        } else if stl_type_str == "QEST01" {
            StlType::Quest
        } else {
            return Err(anyhow!("Invalid STL type: {}", stl_type_str));
        };

        let key_count = reader.read_u32()? as usize;
        let mut string_keys = HashMap::with_capacity(key_count);
        let mut integer_keys = HashMap::with_capacity(key_count);
        for i in 0..key_count {
            let key = reader.read_variable_length_string()?;
            let index = reader.read_u32()?;
            string_keys.insert(key.to_string(), i as u32);
            integer_keys.insert(index, i as u32);
        }

        let language_count = reader.read_u32()? as usize;
        let mut data = Vec::new();

        let read_stl_entry = |reader: &mut RoseFileReader,
                              data: &mut Vec<u8>|
         -> Result<(u32, u32), anyhow::Error> {
            let text_bytes = reader.read_variable_length_bytes()?;
            let _ = str::from_utf8(text_bytes)?;
            let text_bytes_length = text_bytes.len();
            let text_offset = data.len();
            data.extend_from_slice(text_bytes);
            Ok((text_offset as u32, text_bytes_length as u32))
        };

        let mut languages = Vec::with_capacity(language_count);
        for language_index in 0..language_count {
            let language_offset = reader.read_u32()?;
            let language_save_position = reader.position();
            let mut language = StlLanguage::with_capacity(key_count);
            reader.set_position(language_offset as u64);

            if let Some(language_filter) = read_options.language_filter.as_ref() {
                if !language_filter.contains(&language_index) {
                    languages.push(language);
                    reader.set_position(language_save_position);
                    continue;
                }
            }

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
}

impl StlFile {
    pub fn keys(&self) -> Keys<'_, String, u32> {
        self.string_keys.keys()
    }

    pub fn lookup_key(&self, key: &str) -> Option<usize> {
        self.string_keys.get(key).map(|x| *x as usize)
    }

    pub fn get_text_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.text.get(*index as usize)?;
        unsafe {
            Some(str::from_utf8_unchecked(
                &self.data[*offset as usize..(offset + size) as usize],
            ))
        }
    }

    pub fn get_comment_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.comment.get(*index as usize)?;
        unsafe {
            Some(str::from_utf8_unchecked(
                &self.data[*offset as usize..(offset + size) as usize],
            ))
        }
    }

    pub fn get_quest1_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.quest1.get(*index as usize)?;
        unsafe {
            Some(str::from_utf8_unchecked(
                &self.data[*offset as usize..(offset + size) as usize],
            ))
        }
    }

    pub fn get_quest2_string(&self, language: usize, key: &str) -> Option<&str> {
        let language = self.languages.get(language)?;
        let index = self.string_keys.get(key)?;
        let (offset, size) = language.quest2.get(*index as usize)?;
        unsafe {
            Some(str::from_utf8_unchecked(
                &self.data[*offset as usize..(offset + size) as usize],
            ))
        }
    }

    pub fn get_normal_entry(&self, language: usize, index: usize) -> Option<StlNormalEntry<'_>> {
        let language = self.languages.get(language)?;

        Some(StlNormalEntry {
            text: {
                let (offset, size) = language.text.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
        })
    }

    pub fn get_item_entry(&self, language: usize, index: usize) -> Option<StlItemEntry<'_>> {
        let language = self.languages.get(language)?;

        Some(StlItemEntry {
            name: {
                let (offset, size) = language.text.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
            description: {
                let (offset, size) = language.comment.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
        })
    }

    pub fn get_quest_entry(&self, language: usize, index: usize) -> Option<StlQuestEntry<'_>> {
        let language = self.languages.get(language)?;

        Some(StlQuestEntry {
            name: {
                let (offset, size) = language.text.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
            description: {
                let (offset, size) = language.comment.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
            start_message: {
                let (offset, size) = language.quest1.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
            end_message: {
                let (offset, size) = language.quest2.get(index)?;
                unsafe {
                    str::from_utf8_unchecked(&self.data[*offset as usize..(offset + size) as usize])
                }
            },
        })
    }
}
