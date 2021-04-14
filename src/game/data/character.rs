use super::STB_INIT_AVATAR;
use crate::game::components::{
    BasicStats, CharacterDeleteTime, CharacterInfo, Equipment, Inventory, Level, Position,
};
use crate::game::data::CHARACTER_STORAGE_DIR;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf};

pub enum CharacterStorageError {
    NotFound,
    IoError,
    InvalidValue,
}

impl From<std::io::Error> for CharacterStorageError {
    fn from(_: std::io::Error) -> Self {
        CharacterStorageError::IoError
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
    pub delete_time: Option<CharacterDeleteTime>,
}

fn get_character_path(name: &str) -> PathBuf {
    CHARACTER_STORAGE_DIR.join(format!("{}.json", name))
}

impl CharacterStorage {
    pub fn new(
        name: String,
        gender: u8,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<Self, CharacterStorageError> {
        let init_avatar_row = gender as usize;
        if init_avatar_row >= STB_INIT_AVATAR.rows() {
            return Err(CharacterStorageError::InvalidValue);
        }

        // TODO: Verify birth_stone, face, hair values

        let mut character = Self {
            info: CharacterInfo {
                name: name,
                gender: gender,
                birth_stone,
                job: 0,
                face: face,
                hair: hair,
            },
            basic_stats: STB_INIT_AVATAR.get_basic_stats(init_avatar_row),
            equipment: Equipment::default(),
            inventory: Inventory::default(),
            level: Level::default(),
            position: Position {
                x: 530500f32,
                y: 539500f32,
                z: 0,
                zone: 20,
                respawn_zone: 20,
            },
            delete_time: None,
        };
        character
            .equipment
            .equip_items(STB_INIT_AVATAR.get_equipment(init_avatar_row));
        character
            .inventory
            .add_items(STB_INIT_AVATAR.get_inventory_consumables(init_avatar_row));
        character
            .inventory
            .add_items(STB_INIT_AVATAR.get_inventory_equipment(init_avatar_row));
        character
            .inventory
            .add_items(STB_INIT_AVATAR.get_inventory_materials(init_avatar_row));
        Ok(character)
    }

    pub fn try_create(
        name: String,
        gender: u8,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<Self, CharacterStorageError> {
        let character = Self::new(name, gender, birth_stone, face, hair)?;
        character.save_character_impl(false)?;
        Ok(character)
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