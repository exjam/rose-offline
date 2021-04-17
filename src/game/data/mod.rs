pub mod account;
pub mod character;
pub mod formats;
pub mod items;
pub mod stb;

mod calculate_ability_values;
pub use calculate_ability_values::calculate_ability_values;

use directories::ProjectDirs;
use formats::FileReader;
use formats::StbFile;
use formats::VfsIndex;
use lazy_static::lazy_static;
use stb::{StbInitAvatar, StbItem, StbItemBack, StbItemFoot, StbZone};
use std::path::Path;
use std::path::PathBuf;

use self::stb::StbSkill;

fn load_stb(path: &str) -> StbFile {
    if let Some(file) = VFS_INDEX.open_file(path) {
        if let Ok(data) = StbFile::read(FileReader::from(&file)) {
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
    pub static ref STB_INIT_AVATAR: StbInitAvatar =
        StbInitAvatar(load_stb("3DDATA/STB/INIT_AVATAR.STB"));
    pub static ref STB_SKILL: StbSkill = StbSkill(load_stb("3DDATA/STB/LIST_SKILL.STB"));
    pub static ref STB_ZONE: StbZone = StbZone(load_stb("3DDATA/STB/LIST_ZONE.STB"));
    pub static ref STB_HAIR: StbItem = StbItem(load_stb("3DDATA/STB/LIST_HAIR.STB"));
    pub static ref STB_FACE: StbItem = StbItem(load_stb("3DDATA/STB/LIST_FACE.STB"));
    pub static ref STB_ITEM_FACE: StbItem = StbItem(load_stb("3DDATA/STB/LIST_FACEITEM.STB"));
    pub static ref STB_ITEM_BODY: StbItem = StbItem(load_stb("3DDATA/STB/LIST_BODY.STB"));
    pub static ref STB_ITEM_HANDS: StbItem = StbItem(load_stb("3DDATA/STB/LIST_ARMS.STB"));
    pub static ref STB_ITEM_HEAD: StbItem = StbItem(load_stb("3DDATA/STB/LIST_CAP.STB"));
    pub static ref STB_ITEM_FEET: StbItemFoot =
        StbItemFoot(StbItem(load_stb("3DDATA/STB/LIST_FOOT.STB")));
    pub static ref STB_ITEM_BACK: StbItemBack =
        StbItemBack(StbItem(load_stb("3DDATA/STB/LIST_BACK.STB")));
    pub static ref STB_ITEM_JEWELLERY: StbItem = StbItem(load_stb("3DDATA/STB/LIST_JEWEL.STB"));
    pub static ref STB_ITEM_WEAPON: StbItem = StbItem(load_stb("3DDATA/STB/LIST_WEAPON.STB"));
    pub static ref STB_ITEM_SUB_WEAPON: StbItem = StbItem(load_stb("3DDATA/STB/LIST_SUBWPN.STB"));
    pub static ref STB_ITEM_CONSUMABLE: StbItem = StbItem(load_stb("3DDATA/STB/LIST_USEITEM.STB"));
    pub static ref STB_ITEM_GEM: StbItem = StbItem(load_stb("3DDATA/STB/LIST_JEMITEM.STB"));
    pub static ref STB_ITEM_MATERIAL: StbItem = StbItem(load_stb("3DDATA/STB/LIST_NATURAL.STB"));
    pub static ref STB_ITEM_QUEST: StbItem = StbItem(load_stb("3DDATA/STB/LIST_QUESTITEM.STB"));
    pub static ref STB_ITEM_VEHICLE: StbItem = StbItem(load_stb("3DDATA/STB/LIST_PAT.STB"));
}
