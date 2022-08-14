use std::{num::NonZeroUsize, sync::Arc, time::Duration};

use rose_data::{EffectData, EffectDatabase, EffectFileId, EffectId, SoundId};
use rose_file_readers::{stb_column, StbFile, VfsPathBuf, VirtualFilesystem};

use crate::data_decoder::IroseEffectBulletMoveType;

pub struct StbEffect(pub StbFile);

#[allow(dead_code)]
impl StbEffect {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { (2..=5), get_effect_points, [Option<EffectFileId>; 4] }

    stb_column! { 6, get_trail_colour_index, NonZeroUsize }
    stb_column! { 8, get_trail_duration_millis, u32 }

    stb_column! { 9, get_hit_normal, EffectFileId }
    stb_column! { 10, get_hit_critical, EffectFileId }

    stb_column! { 11, get_bullet_normal, EffectFileId }
    stb_column! { 13, get_bullet_move_type, IroseEffectBulletMoveType }
    stb_column! { 15, get_bullet_speed, f32 }

    stb_column! { 16, get_fire_sound_id, SoundId }
    stb_column! { 17, get_hit_sound_id, SoundId }
}

fn load_effect(data: &StbEffect, id: usize) -> Option<EffectData> {
    Some(EffectData {
        id: EffectId::new(id as u16)?,
        point_effects: data
            .get_effect_points(id)
            .iter()
            .filter_map(|x| x.as_ref())
            .copied()
            .collect(),
        trail_colour_index: data.get_trail_colour_index(id),
        trail_duration: Duration::from_millis(
            data.get_trail_duration_millis(id).unwrap_or(0) as u64
        ),
        hit_effect_normal: data.get_hit_normal(id),
        hit_effect_critical: data.get_hit_critical(id),
        bullet_effect: data.get_bullet_normal(id),
        bullet_move_type: data
            .get_bullet_move_type(id)
            .and_then(|x| x.try_into().ok()),
        bullet_speed: data.get_bullet_speed(id).unwrap_or(0.0),
        fire_sound_id: data.get_fire_sound_id(id),
        hit_sound_id: data.get_hit_sound_id(id),
    })
}

pub fn get_effect_database(vfs: &VirtualFilesystem) -> Result<Arc<EffectDatabase>, anyhow::Error> {
    let stb_effect_files = vfs.read_file::<StbFile, _>("3DDATA/STB/FILE_EFFECT.STB")?;

    let mut effect_files = Vec::new();
    for row in 0..stb_effect_files.rows() {
        let path = stb_effect_files.get(row, 1);
        if path.is_empty() {
            effect_files.push(None);
        } else {
            effect_files.push(Some(VfsPathBuf::new(path)));
        }
    }

    let stb_effects = StbEffect(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_EFFECT.STB")?);
    let mut effects = Vec::new();
    for row in 0..stb_effects.rows() {
        effects.push(load_effect(&stb_effects, row));
    }

    Ok(Arc::new(EffectDatabase::new(effects, effect_files)))
}
