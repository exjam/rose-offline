use std::{io::Write, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use rose_game_common::components::{ClanLevel, ClanMark, ClanMemberPosition, ClanPoints, Money};

use crate::game::storage::CLAN_STORAGE_DIR;

#[derive(Deserialize, Serialize)]
pub struct ClanStorageMember {
    pub name: String,
    pub position: ClanMemberPosition,
    pub contribution: ClanPoints,
}

impl ClanStorageMember {
    pub fn new(name: String, position: ClanMemberPosition) -> Self {
        Self {
            name,
            position,
            contribution: ClanPoints(0),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ClanStorage {
    pub name: String,
    pub description: String,
    pub mark: ClanMark,
    pub money: Money,
    pub points: ClanPoints,
    pub level: ClanLevel,
    pub members: Vec<ClanStorageMember>,
}

fn get_clan_path(name: &str) -> PathBuf {
    CLAN_STORAGE_DIR.join(format!("{}.json", name))
}

impl ClanStorage {
    pub fn new(name: String, description: String, mark: ClanMark) -> Self {
        Self {
            name,
            description,
            mark,
            money: Money(0),
            points: ClanPoints(0),
            level: ClanLevel(1),
            members: Vec::default(),
        }
    }

    pub fn exists(name: &str) -> bool {
        get_clan_path(name).exists()
    }

    pub fn try_create(&self) -> Result<(), anyhow::Error> {
        self.save_clan_impl(false)
    }

    pub fn try_load(name: &str) -> Result<Self, anyhow::Error> {
        let path = get_clan_path(name);
        let str = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.to_string_lossy()))?;
        let clan: Self = serde_json::from_str(&str).with_context(|| {
            format!(
                "Failed to deserialise ClanStorage from file {}",
                path.to_string_lossy()
            )
        })?;
        Ok(clan)
    }

    pub fn try_load_clan_list() -> Result<Vec<Self>, anyhow::Error> {
        let mut clan_list = Vec::new();

        for entry in (CLAN_STORAGE_DIR.read_dir()?).flatten() {
            let path = entry.path();
            let str = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file {}", path.to_string_lossy()))?;
            let clan: Self = serde_json::from_str(&str).with_context(|| {
                format!(
                    "Failed to deserialise ClanStorage from file {}",
                    path.to_string_lossy()
                )
            })?;
            clan_list.push(clan);
        }

        Ok(clan_list)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.save_clan_impl(true)
    }

    fn save_clan_impl(&self, allow_overwrite: bool) -> Result<(), anyhow::Error> {
        let path = get_clan_path(&self.name);
        let storage_dir = path.parent().unwrap();

        std::fs::create_dir_all(storage_dir).with_context(|| {
            format!(
                "Failed to create clan storage directory {}",
                storage_dir.to_string_lossy()
            )
        })?;

        let json = serde_json::to_string_pretty(self).with_context(|| {
            format!(
                "Failed to serialise ClanStorage whilst saving clan {}",
                &self.name
            )
        })?;
        let mut file = tempfile::NamedTempFile::new().with_context(|| {
            format!(
                "Failed to create temporary file whilst saving clan {}",
                &self.name
            )
        })?;
        file.write_all(json.as_bytes()).with_context(|| {
            format!(
                "Failed to write data to temporary file whilst saving clan {}",
                &self.name
            )
        })?;

        if allow_overwrite {
            file.persist(&path).with_context(|| {
                format!(
                    "Failed to persist temporary clan file to path {}",
                    path.to_string_lossy()
                )
            })?;
        } else {
            file.persist_noclobber(&path).with_context(|| {
                format!(
                    "Failed to persist_noclobber temporary clan file to path {}",
                    path.to_string_lossy()
                )
            })?;
        }

        Ok(())
    }
}
