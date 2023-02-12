use bevy::ecs::prelude::Component;

use crate::game::storage::account::AccountStorage;

#[derive(Component)]
pub struct Account {
    pub name: String,
    pub password_md5_sha256: String,
    pub character_names: Vec<String>,
}

impl From<&Account> for AccountStorage {
    fn from(account: &Account) -> Self {
        Self {
            name: account.name.clone(),
            password_md5_sha256: account.password_md5_sha256.clone(),
            character_names: account.character_names.clone(),
        }
    }
}

impl From<AccountStorage> for Account {
    fn from(storage: AccountStorage) -> Self {
        Self {
            name: storage.name,
            password_md5_sha256: storage.password_md5_sha256,
            character_names: storage.character_names,
        }
    }
}
