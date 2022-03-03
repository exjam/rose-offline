mod reader;
pub use reader::RoseFileReader;

pub trait RoseFile {
    type ReadOptions: Default;

    fn read(reader: RoseFileReader, options: &Self::ReadOptions) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub mod types;

mod aip;
mod chr;
mod him;
mod ifo;
mod lit;
mod qsd;
mod stl;
mod til;
mod vfs;
mod zmo;
mod zms;
mod zon;
mod zsc;

#[macro_use]
mod stb;

pub use aip::*;
pub use chr::ChrFile;
pub use him::HimFile;
pub use ifo::{
    IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc, IfoObject,
    IfoReadOptions,
};
pub use lit::{LitFile, LitObject, LitObjectPart};
pub use qsd::*;
pub use stb::{StbFile, StbReadOptions};
pub use stl::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry};
pub use til::TilFile;
pub use vfs::{VfsFile, VfsIndex, VfsPath};
pub use zmo::ZmoFile;
pub use zms::{ZmsFile, ZmsReadError};
pub use zon::{ZonFile, ZonTileRotation};
pub use zsc::{
    ZscCollisionFlags, ZscCollisionShape, ZscEffectType, ZscFile, ZscMaterial, ZscMaterialBlend,
    ZscMaterialGlow, ZscObject, ZscObjectEffect, ZscObjectPart,
};
