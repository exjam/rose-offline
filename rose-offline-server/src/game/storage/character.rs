use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf};

use rose_game_common::components::CharacterGender;

use crate::game::{
    components::{
        BasicStats, CharacterDeleteTime, CharacterInfo, Equipment, ExperiencePoints, HealthPoints,
        Hotbar, Inventory, Level, ManaPoints, Position, QuestState, SkillList, SkillPoints,
        Stamina, StatPoints, UnionMembership,
    },
    storage::CHARACTER_STORAGE_DIR,
};

#[derive(Deserialize, Serialize)]
pub struct CharacterStorage {
    pub info: CharacterInfo,
    pub basic_stats: BasicStats,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub position: Position,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub delete_time: Option<CharacterDeleteTime>,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub skill_points: SkillPoints,
    pub stat_points: StatPoints,
    pub quest_state: QuestState,
    pub union_membership: UnionMembership,
    pub stamina: Stamina,
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
        gender: CharacterGender,
        birth_stone: u8,
        face: u8,
        hair: u8,
    ) -> Result<CharacterStorage, CharacterCreatorError>;

    fn get_basic_stats(&self, gender: CharacterGender)
        -> Result<BasicStats, CharacterCreatorError>;
}

impl CharacterStorage {
    pub fn try_create(&self) -> Result<(), anyhow::Error> {
        self.save_character_impl(false)
    }

    pub fn try_load(name: &str) -> Result<Self, anyhow::Error> {
        let path = get_character_path(name);
        let str = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.to_string_lossy()))?;
        let character: CharacterStorage = serde_json::from_str(&str).with_context(|| {
            format!(
                "Failed to deserialise CharacterStorage from file {}",
                path.to_string_lossy()
            )
        })?;
        Ok(character)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.save_character_impl(true)
    }

    fn save_character_impl(&self, allow_overwrite: bool) -> Result<(), anyhow::Error> {
        let path = get_character_path(&self.info.name);
        let storage_dir = path.parent().unwrap();

        std::fs::create_dir_all(storage_dir).with_context(|| {
            format!(
                "Failed to create character storage directory {}",
                storage_dir.to_string_lossy()
            )
        })?;

        let json = serde_json::to_string_pretty(self).with_context(|| {
            format!(
                "Failed to serialise CharacterStorage whilst saving character {}",
                &self.info.name
            )
        })?;
        let mut file = tempfile::NamedTempFile::new().with_context(|| {
            format!(
                "Failed to create temporary file whilst saving character {}",
                &self.info.name
            )
        })?;
        file.write_all(json.as_bytes()).with_context(|| {
            format!(
                "Failed to write data to temporary file whilst saving character {}",
                &self.info.name
            )
        })?;

        if allow_overwrite {
            file.persist(&path).with_context(|| {
                format!(
                    "Failed to persist temporary character file to path {}",
                    path.to_string_lossy()
                )
            })?;
        } else {
            file.persist_noclobber(&path).with_context(|| {
                format!(
                    "Failed to persist_noclobber temporary character file to path {}",
                    path.to_string_lossy()
                )
            })?;
        }

        Ok(())
    }

    pub fn exists(name: &str) -> bool {
        get_character_path(name).exists()
    }

    pub fn delete(name: &str) -> Result<(), anyhow::Error> {
        let path = get_character_path(name);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }
}
