mod reader;
pub use reader::FileReader;

pub trait RoseFile {
    fn read(reader: FileReader) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub mod types;

pub mod aip;
pub mod chr;
pub mod ifo;
pub mod qsd;
pub mod stl;
pub mod vfs;
pub mod zmo;
pub mod zms;
pub mod zon;

#[macro_use]
pub mod stb;

pub use aip::*;
pub use chr::{ChrFile, ChrReadError};
pub use ifo::{IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc, IfoObject};
pub use qsd::*;
pub use stb::{StbFile, StbReadError};
pub use stl::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry, StlReadError};
pub use vfs::{VfsFile, VfsIndex, VfsPath};
pub use zmo::{ZmoFile, ZmoReadError};
pub use zms::{ZmsFile, ZmsReadError};
pub use zon::{ZonFile, ZonReadError, ZonTileRotation};
