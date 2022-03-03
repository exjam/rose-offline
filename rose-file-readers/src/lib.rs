mod reader;
pub use reader::FileReader;

pub trait RoseFile {
    type ReadOptions: Default;

    fn read(reader: FileReader, options: &Self::ReadOptions) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub mod types;

mod aip;
mod chr;
mod him;
mod ifo;
mod qsd;
mod stl;
mod til;
mod vfs;
mod zmo;
mod zms;
mod zon;

#[macro_use]
mod stb;

pub use aip::*;
pub use chr::{ChrFile, ChrReadError};
pub use him::HimFile;
pub use ifo::{
    IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc, IfoObject,
    IfoReadOptions,
};
pub use qsd::*;
pub use stb::{StbFile, StbReadError, StbReadOptions};
pub use stl::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry, StlReadError};
pub use til::TilFile;
pub use vfs::{VfsFile, VfsIndex, VfsPath};
pub use zmo::{ZmoFile, ZmoReadError};
pub use zms::{ZmsFile, ZmsReadError};
pub use zon::{ZonFile, ZonReadError, ZonTileRotation};
