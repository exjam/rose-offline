use std::sync::Arc;

use rose_data::EffectDatabase;
use rose_file_readers::{StbFile, VfsIndex, VfsPathBuf};

pub fn get_effect_database(vfs: &VfsIndex) -> Option<Arc<EffectDatabase>> {
    let stb_effects = vfs
        .read_file::<StbFile, _>("3DDATA/STB/FILE_EFFECT.STB")
        .ok()?;

    let mut effects = Vec::new();
    for row in 0..stb_effects.rows() {
        let path = stb_effects.get(row, 1);
        if path.is_empty() {
            effects.push(None);
        } else {
            effects.push(Some(VfsPathBuf::new(path)));
        }
    }

    Some(Arc::new(EffectDatabase::new(effects)))
}
