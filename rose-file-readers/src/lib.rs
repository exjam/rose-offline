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
mod con_;
mod eft;
mod him;
mod ifo;
mod lit;
mod ltb;
mod ptl;
mod qsd;
mod stl;
mod til;
mod vfs;
mod zmd;
mod zmo;
mod zms;
mod zon;
mod zsc;

#[macro_use]
mod stb;

pub use aip::*;
pub use chr::ChrFile;
pub use con_::{ConFile, ConMenu, ConMessage, ConMessageType};
pub use eft::{EftFile, EftMesh, EftParticle};
pub use him::HimFile;
pub use ifo::{
    IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc, IfoObject,
    IfoReadOptions,
};
pub use lit::{LitFile, LitObject, LitObjectPart};
pub use ltb::LtbFile;
pub use ptl::{PtlFile, PtlKeyframe, PtlKeyframeData, PtlSequence};
pub use qsd::*;
pub use stb::{StbFile, StbReadOptions};
pub use stl::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry, StlReadOptions};
pub use til::TilFile;
pub use vfs::{VfsFile, VfsIndex, VfsPath, VfsPathBuf};
pub use zmd::ZmdFile;
pub use zmo::{ZmoChannel, ZmoFile, ZmoReadOptions};
pub use zms::{ZmsFile, ZmsReadError};
pub use zon::{ZonFile, ZonReadOptions, ZonTile, ZonTileRotation};
pub use zsc::{
    ZscCollisionFlags, ZscCollisionShape, ZscEffectType, ZscFile, ZscMaterial, ZscMaterialBlend,
    ZscMaterialGlow, ZscObject, ZscObjectEffect, ZscObjectPart,
};
