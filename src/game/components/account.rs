use crate::game::LOCAL_STORAGE_DIR;
use hex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;

pub enum AccountError {
    Failed,
    NotFound,
    InvalidPassword,
    MaxCharacters,
    IoError,
}

impl From<std::io::Error> for AccountError {
    fn from(_: std::io::Error) -> AccountError {
        AccountError::IoError
    }
}

impl From<serde_json::Error> for AccountError {
    fn from(_: serde_json::Error) -> AccountError {
        AccountError::IoError
    }
}

impl From<tempfile::PersistError> for AccountError {
    fn from(_: tempfile::PersistError) -> AccountError {
        AccountError::IoError
    }
}

#[derive(Deserialize, Serialize)]
pub struct Account {
    pub name: String,
    pub password_md5_sha256: String,
    pub character_names: Vec<String>,
}

fn get_account_path(name: &str) -> PathBuf {
    LOCAL_STORAGE_DIR
        .join("accounts")
        .join(format!("{}.json", name))
}

fn hash_md5_password(password_md5: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password_md5);
    hex::encode(hasher.finalize())
}

impl Account {
    pub fn try_load(name: &str, password_md5: &str) -> Result<Account, AccountError> {
        let path = get_account_path(name);
        if path.exists() {
            let str = std::fs::read_to_string(path)?;
            let account: Account = serde_json::from_str(&str)?;
            account.check_password(password_md5)?;
            Ok(account)
        } else {
            let account = Account {
                name: String::from(name),
                password_md5_sha256: hash_md5_password(password_md5),
                character_names: Vec::new(),
            };
            account.save_impl(false)?;
            Ok(account)
        }
    }

    pub fn check_password(&self, password_md5: &str) -> Result<(), AccountError> {
        if self.password_md5_sha256 == hash_md5_password(password_md5) {
            Ok(())
        } else {
            Err(AccountError::InvalidPassword)
        }
    }

    pub fn save(&self) -> Result<(), AccountError> {
        self.save_impl(true)
    }

    fn save_impl(&self, allow_overwrite: bool) -> Result<(), AccountError> {
        let path = get_account_path(&self.name);
        if std::fs::create_dir_all(path.parent().unwrap()).is_err() {
            return Err(AccountError::Failed);
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
