use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf};

use crate::data::CHARACTER_STORAGE_DIR;
use crate::game::components::{
    BasicStats, CharacterDeleteTime, CharacterInfo, Equipment, HealthPoints, Hotbar, Inventory,
    Level, ManaPoints, Position, SkillList,
};

pub enum CharacterStorageError {
    NotFound,
    IoError,
}

impl From<std::io::Error> for CharacterStorageError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            CharacterStorageError::NotFound
        } else {
            CharacterStorageError::IoError
        }
    }
}

impl From<serde_json::Error> for CharacterStorageError {
    fn from(_: serde_json::Error) -> Self {
        CharacterStorageError::IoError
    }
}

impl From<tempfile::PersistError> for CharacterStorageError {
    fn from(_: tempfile::PersistError) -> Self {
        CharacterStorageError::IoError
    }
}

#[derive(Deserialize, Serialize)]
pub struct CharacterStorage {
    pub info: CharacterInfo,
    pub basic_stats: BasicStats,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub level: Level,
    pub position: Position,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub delete_time: Option<CharacterDeleteTime>,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
}

fn get_character_path(name: &str) -> PathBuf {
    CHARACTER_STORAGE_DIR.join(format!("{}.json", name))
}

#[allow(dead_code)]
pub enum CharacterCreatorError {
    InvalidName,
    InvalidGender,
    InvalidBirthStone,
    InvalidFace,
    InvalidHair,
}

pub trait CharacterCreator {
    fn create(
        &self,
        name: String,
        gender: u8,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<CharacterStorage, CharacterCreatorError>;
}

impl CharacterStorage {
    pub fn try_create(&self) -> Result<(), CharacterStorageError> {
        self.save_character_impl(false)
    }

    pub fn try_load(name: &str) -> Result<Self, CharacterStorageError> {
        let path = get_character_path(name);
        let str = std::fs::read_to_string(path)?;
        let character: CharacterStorage = serde_json::from_str(&str)?;
        Ok(character)
    }

    pub fn save(&self) -> Result<(), CharacterStorageError> {
        self.save_character_impl(true)
    }

    fn save_character_impl(&self, allow_overwrite: bool) -> Result<(), CharacterStorageError> {
        let path = get_character_path(&self.info.name);

        if std::fs::create_dir_all(path.parent().unwrap()).is_err() {
            return Err(CharacterStorageError::IoError);
        }

        let json = serde_json::to_string_pretty(self)?;
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(json.as_bytes())?;

        if allow_overwrite {
            file.persist(path)?;
        } else {
            file.persist_noclobber(path)?;
        }
        Ok(())
    }

    pub fn exists(name: &str) -> bool {
        get_character_path(name).exists()
    }

    pub fn delete(name: &str) -> Result<(), CharacterStorageError> {
        let path = get_character_path(name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
