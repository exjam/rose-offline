use crate::game::data::ACCOUNT_STORAGE_DIR;
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;

pub enum AccountStorageError {
    Failed,
    NotFound,
    InvalidPassword,
    MaxCharacters,
    IoError,
}

impl From<std::io::Error> for AccountStorageError {
    fn from(_: std::io::Error) -> AccountStorageError {
        AccountStorageError::IoError
    }
}

impl From<serde_json::Error> for AccountStorageError {
    fn from(_: serde_json::Error) -> AccountStorageError {
        AccountStorageError::IoError
    }
}

impl From<tempfile::PersistError> for AccountStorageError {
    fn from(_: tempfile::PersistError) -> AccountStorageError {
        AccountStorageError::IoError
    }
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

fn hash_md5_password(password_md5: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password_md5);
    hex::encode(hasher.finalize())
}

impl AccountStorage {
    pub fn try_load(name: &str, password_md5: &str) -> Result<Self, AccountStorageError> {
        let path = get_account_path(name);
        if path.exists() {
            let str = std::fs::read_to_string(path)?;
            let account: Self = serde_json::from_str(&str)?;
            account.check_password(password_md5)?;
            Ok(account)
        } else {
            let account = Self {
                name: String::from(name),
                password_md5_sha256: hash_md5_password(password_md5),
                character_names: Vec::new(),
            };
            account.save_impl(false)?;
            Ok(account)
        }
    }

    pub fn check_password(&self, password_md5: &str) -> Result<(), AccountStorageError> {
        if self.password_md5_sha256 == hash_md5_password(password_md5) {
            Ok(())
        } else {
            Err(AccountStorageError::InvalidPassword)
        }
    }

    pub fn save(&self) -> Result<(), AccountStorageError> {
        self.save_impl(true)
    }

    fn save_impl(&self, allow_overwrite: bool) -> Result<(), AccountStorageError> {
        let path = get_account_path(&self.name);
        if std::fs::create_dir_all(path.parent().unwrap()).is_err() {
            return Err(AccountStorageError::Failed);
        }

        let json = serde_json::to_string_pretty(&self)?;
        let mut file = tempfile::NamedTempFile::new()?;
        file.write_all(json.as_bytes())?;
        if allow_overwrite {
            file.persist(path)?;
        } else {
            file.persist_noclobber(path)?;
        }
        Ok(())
    }
}
