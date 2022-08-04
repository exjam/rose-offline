use std::sync::Arc;

use rose_data::{EffectFileId, SoundData, SoundDatabase, SoundId};
use rose_file_readers::{stb_column, StbFile, VfsPathBuf, VirtualFilesystem};

pub struct StbSound(pub StbFile);

#[allow(dead_code)]
impl StbSound {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { (2..=5), get_effect_points, [Option<EffectFileId>; 4] }

    stb_column! { 0, get_path, &str }
    stb_column! { 1, get_max_mix_count, u32 }
}

fn load_sound(data: &StbSound, id: usize) -> Option<SoundData> {
    Some(SoundData {
        id: SoundId::new(id as u16)?,
        path: VfsPathBuf::new(data.get_path(id)?),
        max_mix_count: data.get_max_mix_count(id).unwrap_or(0) as usize,
    })
}

pub fn get_sound_database(vfs: &VirtualFilesystem) -> Result<Arc<SoundDatabase>, anyhow::Error> {
    let stb_sounds = StbSound(vfs.read_file::<StbFile, _>("3DDATA/STB/FILE_SOUND.STB")?);
    let mut sounds = Vec::new();
    for row in 0..stb_sounds.rows() {
        sounds.push(load_sound(&stb_sounds, row));
    }

    let stb_step_sounds = vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_STEPSOUND.STB")?;
    let mut step_sounds = Vec::with_capacity(stb_step_sounds.rows() * stb_step_sounds.columns());
    let step_sound_zone_types = stb_step_sounds.columns();
    for row in 0..stb_step_sounds.rows() {
        for col in 0..stb_step_sounds.columns() {
            step_sounds.push(SoundId::new(stb_step_sounds.get_int(row, col) as u16));
        }
    }

    let stb_hit_sounds = vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_HITSOUND.STB")?;
    let mut hit_sounds = Vec::with_capacity(stb_hit_sounds.rows() * stb_hit_sounds.columns());
    let hit_sound_material_types = stb_hit_sounds.columns();
    for row in 0..stb_hit_sounds.rows() {
        for col in 0..stb_hit_sounds.columns() {
            hit_sounds.push(SoundId::new(stb_hit_sounds.get_int(row, col) as u16));
        }
    }

    Ok(Arc::new(SoundDatabase::new(
        sounds,
        step_sounds,
        step_sound_zone_types,
        hit_sounds,
        hit_sound_material_types,
    )))
}
