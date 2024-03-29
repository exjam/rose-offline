use rose_file_readers::VfsPathBuf;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct SoundId(NonZeroU16);

id_wrapper_impl!(SoundId, NonZeroU16, u16);

#[derive(Debug)]
pub struct SoundData {
    pub id: SoundId,
    pub path: VfsPathBuf,
    pub max_mix_count: usize,
}

#[derive(Debug)]
pub struct SoundDatabase {
    sounds: Vec<Option<SoundData>>,
    step_sounds: Vec<Option<SoundId>>,
    step_sound_zone_types: usize,
    hit_sounds: Vec<Option<SoundId>>,
    hit_sound_material_types: usize,
}

impl SoundDatabase {
    pub fn new(
        sounds: Vec<Option<SoundData>>,
        step_sounds: Vec<Option<SoundId>>,
        step_sound_zone_types: usize,
        hit_sounds: Vec<Option<SoundId>>,
        hit_sound_material_types: usize,
    ) -> Self {
        Self {
            sounds,
            step_sounds,
            step_sound_zone_types,
            hit_sounds,
            hit_sound_material_types,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SoundData> {
        self.sounds.iter().filter_map(|x| x.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.sounds.is_empty()
    }

    pub fn len(&self) -> usize {
        self.sounds.len()
    }

    pub fn get_sound(&self, id: SoundId) -> Option<&SoundData> {
        self.sounds.get(id.get() as usize).and_then(|x| x.as_ref())
    }

    pub fn get_hit_sound(
        &self,
        weapon_hit_sound_type: usize,
        material_type: usize,
    ) -> Option<&SoundData> {
        self.hit_sounds
            .get(material_type + weapon_hit_sound_type * self.hit_sound_material_types)
            .and_then(|id| *id)
            .and_then(|id| self.get_sound(id))
    }

    pub fn get_step_sound(&self, tile_number: usize, zone_type: usize) -> Option<&SoundData> {
        self.step_sounds
            .get(zone_type + tile_number * self.step_sound_zone_types)
            .and_then(|id| *id)
            .and_then(|id| self.get_sound(id))
    }
}
