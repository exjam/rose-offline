use std::{sync::Arc, time::Duration};

use rose_data::{EffectData, EffectDatabase, EffectFileId, SoundId};
use rose_file_readers::{stb_column, StbFile, VfsIndex, VfsPathBuf};

use crate::data_decoder::IroseEffectBulletMoveType;

pub struct StbEffect(pub StbFile);

#[allow(dead_code)]
impl StbEffect {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { (2..=5), get_effect_points, [Option<EffectFileId>; 4] }

    stb_column! { 6, get_trail_normal, EffectFileId }
    stb_column! { 7, get_trail_critical, EffectFileId }
    stb_column! { 8, get_trail_duration, u32 }

    stb_column! { 9, get_hit_normal, EffectFileId }
    stb_column! { 10, get_hit_critical, EffectFileId }

    stb_column! { 11, get_bullet_normal, EffectFileId }
    stb_column! { 12, get_bullet_critical, EffectFileId }
    stb_column! { 13, get_bullet_move_type, IroseEffectBulletMoveType }
    stb_column! { 15, get_bullet_speed, f32 }

    stb_column! { 16, get_fire_sound_id, SoundId }
    stb_column! { 17, get_hit_sound_id, SoundId }
}

fn load_effect(data: &StbEffect, id: usize) -> Option<EffectData> {
    Some(EffectData {
        point_effects: data
            .get_effect_points(id)
            .iter()
            .filter_map(|x| x.as_ref())
            .copied()
            .collect(),
        trail_normal: data.get_trail_normal(id),
        trail_critical: data.get_trail_critical(id),
        trail_duration: Duration::from_millis(data.get_trail_duration(id).unwrap_or(0) as u64),
        hit_normal: data.get_hit_normal(id),
        hit_critical: data.get_hit_critical(id),
        bullet_normal: data.get_bullet_normal(id),
        bullet_critical: data.get_bullet_critical(id),
        bullet_move_type: data
            .get_bullet_move_type(id)
            .and_then(|x| x.try_into().ok()),
        bullet_speed: data.get_bullet_speed(id).unwrap_or(0.0),
        fire_sound_id: data.get_fire_sound_id(id),
        hit_sound_id: data.get_hit_sound_id(id),
    })
}

pub fn get_effect_database(vfs: &VfsIndex) -> Option<Arc<EffectDatabase>> {
    let stb_effect_files = vfs
        .read_file::<StbFile, _>("3DDATA/STB/FILE_EFFECT.STB")
        .ok()?;

    let mut effect_files = Vec::new();
    for row in 0..stb_effect_files.rows() {
        let path = stb_effect_files.get(row, 1);
        if path.is_empty() {
            effect_files.push(None);
        } else {
            effect_files.push(Some(VfsPathBuf::new(path)));
        }
    }

    let stb_effects = StbEffect(
        vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_EFFECT.STB")
            .ok()?,
    );
    let mut effects = Vec::new();
    for row in 0..stb_effects.rows() {
        effects.push(load_effect(&stb_effects, row));
    }

    Some(Arc::new(EffectDatabase::new(effects, effect_files)))
}
