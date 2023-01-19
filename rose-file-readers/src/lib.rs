mod aruavfs;
mod reader;
mod virtual_filesystem;
mod writer;

pub use reader::RoseFileReader;
pub use writer::RoseFileWriter;

pub trait RoseFile {
    type ReadOptions: Default;
    type WriteOptions: Default;

    fn read(reader: RoseFileReader, options: &Self::ReadOptions) -> Result<Self, anyhow::Error>
    where
        Self: Sized;

    fn write(
        &self,
        _writer: &mut RoseFileWriter,
        _options: &Self::WriteOptions,
    ) -> Result<(), anyhow::Error> {
        unimplemented!();
    }
}

pub mod types;

mod aip;
mod chr;
mod con_;
mod eft;
mod him;
mod id;
mod ifo;
mod lit;
mod ltb;
mod ptl;
mod qsd;
mod stl;
mod til;
mod titanvfs;
mod tsi;
mod vfs;
mod zmd;
mod zmo;
mod zms;
mod zon;
mod zsc;

#[macro_use]
mod stb;

pub use aip::*;
pub use aruavfs::AruaVfsIndex;
pub use chr::ChrFile;
pub use con_::{ConFile, ConMenu, ConMessage, ConMessageType};
pub use eft::{EftFile, EftMesh, EftParticle};
pub use him::HimFile;
pub use id::IdFile;
pub use ifo::{
    IfoEventObject, IfoFile, IfoMonsterSpawn, IfoMonsterSpawnPoint, IfoNpc, IfoObject,
    IfoReadOptions,
};
pub use lit::{LitFile, LitObject, LitObjectPart};
pub use ltb::LtbFile;
pub use ptl::{PtlFile, PtlKeyframe, PtlKeyframeData, PtlSequence, PtlUpdateCoords};
pub use qsd::*;
pub use stb::{StbFile, StbReadOptions};
pub use stl::{StlFile, StlItemEntry, StlNormalEntry, StlQuestEntry, StlReadOptions};
pub use til::TilFile;
pub use titanvfs::TitanVfsIndex;
pub use tsi::{TsiFile, TsiSprite, TsiTexture, TsiTextureId};
pub use vfs::VfsIndex;
pub use virtual_filesystem::{
    HostFilesystemDevice, VfsError, VfsFile, VfsPath, VfsPathBuf, VirtualFilesystem,
    VirtualFilesystemDevice,
};
pub use zmd::ZmdFile;
pub use zmo::{ZmoChannel, ZmoFile, ZmoReadOptions};
pub use zms::{ZmsFile, ZmsReadError};
pub use zon::{ZonFile, ZonReadOptions, ZonTile, ZonTileRotation};
pub use zsc::{
    ZscCollisionFlags, ZscCollisionShape, ZscEffectType, ZscFile, ZscMaterial, ZscMaterialBlend,
    ZscMaterialGlow, ZscObject, ZscObjectEffect, ZscObjectPart,
};

pub mod editor_friendly {
    pub use crate::qsd::editor_friendly::*;
}
