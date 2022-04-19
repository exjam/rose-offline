use std::sync::Arc;

use rose_data::{EffectFileId, SoundData, SoundDatabase, SoundId};
use rose_file_readers::{stb_column, StbFile, VfsIndex, VfsPathBuf};

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

pub fn get_sound_database(vfs: &VfsIndex) -> Option<Arc<SoundDatabase>> {
    let stb_sounds = StbSound(
        vfs.read_file::<StbFile, _>("3DDATA/STB/FILE_SOUND.STB")
            .ok()?,
    );
    let mut sounds = Vec::new();
    for row in 0..stb_sounds.rows() {
        sounds.push(load_sound(&stb_sounds, row));
    }

    Some(Arc::new(SoundDatabase::new(sounds)))
}
