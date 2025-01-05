use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{io::Write, path::PathBuf};
use thiserror::Error;

use rose_game_common::data::Password;

use crate::game::storage::ACCOUNT_STORAGE_DIR;

#[derive(Error, Debug)]
pub enum AccountStorageError {
    #[error("Invalid password")]
    InvalidPassword,

    #[error("Account not found")]
    NotFound,
}

#[derive(Deserialize, Serialize)]
pub struct AccountStorage {
    pub name: String,
    pub password_md5_sha256: String,
    pub character_names: Vec<String>,
}

fn get_account_path(name: &str) -> PathBuf {
    ACCOUNT_STORAGE_DIR.join(format!("{}.json", name))
}

fn hash_password(password: &Password) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.to_md5());
    hex::encode(hasher.finalize())
}

impl AccountStorage {
    pub fn create(name: &str, password: &Password) -> Result<Self, anyhow::Error> {
        let account = Self {
            name: String::from(name),
            password_md5_sha256: hash_password(password),
            character_names: Vec::new(),
        };
        account.save_impl(false)?;
        Ok(account)
    }

    pub fn try_load(name: &str, password: &Password) -> Result<Self, anyhow::Error> {
        let path = get_account_path(name);
        if path.exists() {
            let str = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file {}", path.to_string_lossy()))?;
            let account: Self = serde_json::from_str(&str).with_context(|| {
                format!(
                    "Failed to deserialise AccountStorage from file {}",
                    path.to_string_lossy()
                )
            })?;
            account.check_password(password)?;
            Ok(account)
        } else {
            Err(AccountStorageError::NotFound.into())
        }
    }

    pub fn check_password(&self, password: &Password) -> Result<(), anyhow::Error> {
        if self.password_md5_sha256 == hash_password(password) {
            Ok(())
        } else {
            Err(AccountStorageError::InvalidPassword.into())
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.save_impl(true)
    }

    fn save_impl(&self, allow_overwrite: bool) -> Result<(), anyhow::Error> {
        let path = get_account_path(&self.name);
        let storage_dir = path.parent().unwrap();

        std::fs::create_dir_all(storage_dir).with_context(|| {
            format!(
                "Failed to create account storage directory {}",
                storage_dir.to_string_lossy()
            )
        })?;

        let json = serde_json::to_string_pretty(&self).with_context(|| {
            format!(
                "Failed to serialise AccountStorage whilst saving account {}",
                &self.name
            )
        })?;

        let mut file = tempfile::Builder::new()
            .tempfile_in(storage_dir)
            .with_context(|| {
                format!(
                    "Failed to create temporary file whilst saving account {}",
                    &self.name
                )
            })?;
        file.write_all(json.as_bytes()).with_context(|| {
            format!(
                "Failed to write data to temporary file whilst saving account {}",
                &self.name
            )
        })?;

        if allow_overwrite {
            file.persist(&path).with_context(|| {
                format!(
                    "Failed to persist temporary account file to path {}",
                    path.to_string_lossy()
                )
            })?;
        } else {
            file.persist_noclobber(&path).with_context(|| {
                format!(
                    "Failed to persist_noclobber temporary account file to path {}",
                    path.to_string_lossy()
                )
            })?;
        }

        Ok(())
    }
}
