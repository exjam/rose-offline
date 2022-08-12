use bevy::math::Vec4;
use enum_map::{Enum, EnumMap};
use rose_file_readers::VfsPathBuf;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct SkyboxId(u16);

id_wrapper_impl!(SkyboxId, u16);

#[derive(Enum, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SkyboxState {
    Morning,
    Day,
    Evening,
    Night,
}

pub struct SkyboxData {
    pub id: SkyboxId,
    pub mesh: VfsPathBuf,
    pub texture_day: VfsPathBuf,
    pub texture_night: VfsPathBuf,
    pub lightmap_blend_op: u32,
    pub map_ambient_color: EnumMap<SkyboxState, Vec4>,
    pub character_ambient_color: EnumMap<SkyboxState, Vec4>,
    pub character_diffuse_color: EnumMap<SkyboxState, Vec4>,
}

pub struct SkyboxDatabase {
    skyboxs: Vec<Option<SkyboxData>>,
}

impl SkyboxDatabase {
    pub fn new(skyboxs: Vec<Option<SkyboxData>>) -> Self {
        Self { skyboxs }
    }

    pub fn get_skybox_data(&self, id: SkyboxId) -> Option<&SkyboxData> {
        self.skyboxs.get(id.get() as usize).and_then(|x| x.as_ref())
    }
}
