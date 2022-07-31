use std::sync::Arc;

use bevy::math::Vec4;
use enum_map::EnumMap;
use rose_data::{SkyboxData, SkyboxDatabase, SkyboxId, SkyboxState};
use rose_file_readers::{stb_column, StbFile, VfsIndex, VfsPathBuf};

struct StbSkybox(pub StbFile);

#[allow(dead_code)]
impl StbSkybox {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 0, get_mesh, &str }
    stb_column! { 1, get_texture_day, &str }
    stb_column! { 2, get_texture_night, &str }
    stb_column! { 5, get_lightmap_blend_op, u32 }
    stb_column! { (6..10), get_map_ambient_color, [u32; 4] }
    stb_column! { (10..18).step_by(2), get_character_ambient_color, [u32; 4] }
    stb_column! { (10..18).skip(1).step_by(2), get_character_diffuse_color, [u32; 4] }
}

fn convert_color(color: u32) -> Vec4 {
    let r = color / 1000000;
    let g = (color % 1000000) / 1000;
    let b = color % 1000;

    Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

fn convert_colors(data: &[u32; 4]) -> EnumMap<SkyboxState, Vec4> {
    enum_map::enum_map! {
        SkyboxState::Morning => convert_color(data[0]),
        SkyboxState::Day => convert_color(data[1]),
        SkyboxState::Evening => convert_color(data[2]),
        SkyboxState::Night => convert_color(data[3]),
    }
}

fn load_skybox(data: &StbSkybox, id: usize) -> Option<SkyboxData> {
    Some(SkyboxData {
        id: SkyboxId::new(id as u16),
        mesh: VfsPathBuf::new(data.get_mesh(id)?),
        texture_day: VfsPathBuf::new(data.get_texture_day(id).unwrap_or_default()),
        texture_night: VfsPathBuf::new(data.get_texture_night(id).unwrap_or_default()),
        lightmap_blend_op: data.get_lightmap_blend_op(id).unwrap_or(0),
        map_ambient_color: convert_colors(&data.get_map_ambient_color(id)),
        character_ambient_color: convert_colors(&data.get_character_ambient_color(id)),
        character_diffuse_color: convert_colors(&data.get_character_diffuse_color(id)),
    })
}

pub fn get_skybox_database(vfs: &VfsIndex) -> Result<Arc<SkyboxDatabase>, anyhow::Error> {
    let stb_sky = StbSkybox(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_SKY.STB")?);
    let mut skyboxs = Vec::new();
    for row in 0..stb_sky.rows() {
        skyboxs.push(load_skybox(&stb_sky, row));
    }

    Ok(Arc::new(SkyboxDatabase::new(skyboxs)))
}
