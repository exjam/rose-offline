use rose_file_readers::VfsPathBuf;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr};

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
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
}

impl SoundDatabase {
    pub fn new(sounds: Vec<Option<SoundData>>) -> Self {
        Self { sounds }
    }

    pub fn get_sound(&self, id: SoundId) -> Option<&SoundData> {
        self.sounds.get(id.get() as usize).and_then(|x| x.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = &SoundData> {
        self.sounds.iter().filter_map(|x| x.as_ref())
    }
}
