use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{io::Write, path::PathBuf};
use thiserror::Error;

use rose_data::Item;

use crate::game::storage::BANK_STORAGE_DIR;

#[derive(Error, Debug)]
pub enum BankStorageError {
    #[error("Account not found")]
    NotFound,
}

#[derive(Default, Deserialize, Serialize)]
pub struct BankStorage {
    pub slots: Vec<Option<Item>>,
}

fn get_bank_path(account_name: &str) -> PathBuf {
    BANK_STORAGE_DIR.join(format!("{}.json", account_name))
}

impl BankStorage {
    pub fn create(account_name: &str) -> Result<Self, anyhow::Error> {
        let bank = BankStorage::default();
        bank.save_impl(account_name, false)?;
        Ok(bank)
    }

    pub fn try_load(account_name: &str) -> Result<Self, anyhow::Error> {
        let path = get_bank_path(account_name);
        if path.exists() {
            let str = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file {}", path.to_string_lossy()))?;
            let bank: Self = serde_json::from_str(&str).with_context(|| {
                format!(
                    "Failed to deserialise AccountStorage from file {}",
                    path.to_string_lossy()
                )
            })?;
            Ok(bank)
        } else {
            Err(BankStorageError::NotFound.into())
        }
    }

    pub fn save(&self, account_name: &str) -> Result<(), anyhow::Error> {
        self.save_impl(account_name, true)
    }

    fn save_impl(&self, account_name: &str, allow_overwrite: bool) -> Result<(), anyhow::Error> {
        let path = get_bank_path(account_name);
        let storage_dir = path.parent().unwrap();

        std::fs::create_dir_all(storage_dir).with_context(|| {
            format!(
                "Failed to create bank storage directory {}",
                storage_dir.to_string_lossy()
            )
        })?;

        let json = serde_json::to_string_pretty(&self).with_context(|| {
            format!(
                "Failed to serialise BankStorage whilst saving bank for account {}",
                account_name
            )
        })?;

        let mut file = tempfile::NamedTempFile::new().with_context(|| {
            format!(
                "Failed to create temporary file whilst saving bank for account {}",
                account_name
            )
        })?;
        file.write_all(json.as_bytes()).with_context(|| {
            format!(
                "Failed to write data to temporary file whilst saving bank for account {}",
                account_name
            )
        })?;

        if allow_overwrite {
            file.persist(&path).with_context(|| {
                format!(
                    "Failed to persist temporary bank file to path {}",
                    path.to_string_lossy()
                )
            })?;
        } else {
            file.persist_noclobber(&path).with_context(|| {
                format!(
                    "Failed to persist_noclobber bank account file to path {}",
                    path.to_string_lossy()
                )
            })?;
        }

        Ok(())
    }
}
