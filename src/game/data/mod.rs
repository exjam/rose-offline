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
use stb::{StbInitAvatar, StbItem, StbItemBack, StbItemFoot};
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
    pub static ref STB_HAIR: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_HAIR.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_HAIR.STB");
    };
    pub static ref STB_FACE: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_FACE.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_FACE.STB");
    };
    pub static ref STB_FACE_ITEM: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_FACEITEM.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_FACEITEM.STB");
    };
    pub static ref STB_BODY: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_BODY.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_BODY.STB");
    };
    pub static ref STB_ARMS: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_ARMS.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_ARMS.STB");
    };
    pub static ref STB_FOOT: StbItemFoot = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_FOOT.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItemFoot(StbItem(data));
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_FOOT.STB");
    };
    pub static ref STB_BACK: StbItemBack = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_BACK.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItemBack(StbItem(data));
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_BACK.STB");
    };
    pub static ref STB_JEWEL: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_JEWEL.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_JEWEL.STB");
    };
    pub static ref STB_WEAPON: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_WEAPON.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_WEAPON.STB");
    };
    pub static ref STB_SUBWPN: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_SUBWPN.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_SUBWPN.STB");
    };
    pub static ref STB_USE_ITEM: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_USEITEM.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_USEITEM.STB");
    };
    pub static ref STB_GEM_ITEM: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_JEMITEM.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_JEMITEM.STB");
    };
    pub static ref STB_NATURAL: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_NATURAL.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_NATURAL.STB");
    };
    pub static ref STB_QUEST_ITEM: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_QUESTITEM.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_QUESTITEM.STB");
    };
    pub static ref STB_PAT: StbItem = {
        if let Some(file) = VFS_INDEX.open_file("3DDATA/STB/LIST_PAT.STB") {
            if let Ok(data) = STB::read(FileReader::from(&file)) {
                return StbItem(data);
            }
        }

        panic!("Failed reading 3DDATA/STB/LIST_PAT.STB");
    };
}
