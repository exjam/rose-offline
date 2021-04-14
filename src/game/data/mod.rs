pub mod account;
pub mod character;
pub mod formats;
pub mod items;
pub mod stb;

use directories::ProjectDirs;
use formats::FileReader;
use formats::VfsIndex;
use formats::STB;
use lazy_static::lazy_static;
use stb::StbInitAvatar;
use std::path::Path;
use std::path::PathBuf;

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
    pub static ref STB_INIT_AVATAR: StbInitAvatar = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/INIT_AVATAR.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbInitAvatar(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/INIT_AVATAR.STB");
    };
}
