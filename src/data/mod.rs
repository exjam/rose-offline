pub mod account;
pub mod character;
pub mod formats;
pub mod item;

mod calculate_ability_values;
pub use calculate_ability_values::calculate_ability_values;

use directories::ProjectDirs;
use lazy_static::lazy_static;
use std::path::Path;
use std::path::PathBuf;

use formats::{FileReader, StbFile, VfsIndex};

mod item_database;
mod npc_database;
mod skill_database;
mod zone_database;

pub use character::{CharacterCreator, CharacterCreatorError};
pub use item_database::{
    BackItemData, BaseItemData, BodyItemData, ConsumableItemData, FaceItemData, FeetItemData,
    GemItemData, HandsItemData, HeadItemData, ItemData, ItemDatabase, ItemReference,
    JewelleryItemData, MaterialItemData, QuestItemData, SubWeaponItemData, VehicleItemData,
    WeaponItemData,
};
pub use npc_database::{
    NpcConversationData, NpcConversationReference, NpcData, NpcDatabase, NpcReference,
};
pub use skill_database::{SkillData, SkillDatabase, SkillPage, SkillReference};
pub use zone_database::{ZoneData, ZoneDatabase, ZoneMonsterSpawnPoint, ZoneNpcSpawn};

fn load_stb(path: &str) -> StbFile {
    if let Some(file) = VFS_INDEX.open_file(path) {
        if let Ok(data) = StbFile::read(FileReader::from(&file)) {
            return data;
        }
    }

    panic!("Failed reading {}", path);
}

fn load_stb_with_keys(path: &str) -> StbFile {
    if let Some(file) = VFS_INDEX.open_file(path) {
        if let Ok(data) = StbFile::read_with_keys(FileReader::from(&file)) {
            return data;
        }
    }

    panic!("Failed reading {}", path);
}

lazy_static! {
    pub static ref LOCAL_STORAGE_DIR: PathBuf = {
        let project = ProjectDirs::from("", "", "rose-offline").unwrap();
        PathBuf::from(project.data_local_dir())
    };
    pub static ref ACCOUNT_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("accounts");
    pub static ref CHARACTER_STORAGE_DIR: PathBuf = LOCAL_STORAGE_DIR.join("characters");
    pub static ref VFS_INDEX: VfsIndex = {
        if let Ok(index) = VfsIndex::load(&Path::new("data.idx")) {
            return index;
        }

        panic!("Failed reading data.idx");
    };
}
