use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr, time::Duration};

use rose_file_readers::VfsPathBuf;

use crate::SoundId;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct EffectId(NonZeroU16);

id_wrapper_impl!(EffectId, NonZeroU16, u16);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct EffectFileId(NonZeroU16);

id_wrapper_impl!(EffectFileId, NonZeroU16, u16);

#[derive(Copy, Clone, Debug)]
pub enum EffectBulletMoveType {
    Linear,
    Parabola,
    Immediate,
}

#[derive(Debug)]
pub struct EffectData {
    pub id: EffectId,
    pub point_effects: ArrayVec<EffectFileId, 4>,
    pub trail_effect: Option<EffectFileId>,
    pub trail_duration: Duration,
    pub hit_effect_normal: Option<EffectFileId>,
    pub hit_effect_critical: Option<EffectFileId>,
    pub bullet_effect: Option<EffectFileId>,
    pub bullet_move_type: Option<EffectBulletMoveType>,
    pub bullet_speed: f32,
    pub fire_sound_id: Option<SoundId>,
    pub hit_sound_id: Option<SoundId>,
}

#[derive(Debug)]
pub struct EffectDatabase {
    effects: Vec<Option<EffectData>>,
    effect_files: Vec<Option<VfsPathBuf>>,
}

impl EffectDatabase {
    pub fn new(effects: Vec<Option<EffectData>>, effect_files: Vec<Option<VfsPathBuf>>) -> Self {
        Self {
            effects,
            effect_files,
        }
    }

    pub fn get_effect(&self, id: EffectId) -> Option<&EffectData> {
        self.effects.get(id.get() as usize).and_then(|x| x.as_ref())
    }

    pub fn get_effect_file(&self, id: EffectFileId) -> Option<&VfsPathBuf> {
        self.effect_files
            .get(id.get() as usize)
            .and_then(|x| x.as_ref())
    }

    pub fn iter_files(&self) -> impl Iterator<Item = (EffectFileId, &VfsPathBuf)> {
        self.effect_files
            .iter()
            .enumerate()
            .filter_map(|(id, path)| {
                path.as_ref()
                    .map(|path| (EffectFileId::new(id as u16).unwrap(), path))
            })
    }
}
