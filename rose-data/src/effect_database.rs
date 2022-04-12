use serde::{Deserialize, Serialize};
use std::{num::NonZeroU16, str::FromStr};

use rose_file_readers::VfsPathBuf;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Serialize, Deserialize)]
pub struct EffectId(NonZeroU16);

id_wrapper_impl!(EffectId, NonZeroU16, u16);

pub struct EffectDatabase {
    effects: Vec<Option<VfsPathBuf>>,
}

impl EffectDatabase {
    pub fn new(effects: Vec<Option<VfsPathBuf>>) -> Self {
        Self { effects }
    }

    pub fn get_effect(&self, id: EffectId) -> Option<&VfsPathBuf> {
        self.effects.get(id.get() as usize).and_then(|x| x.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = (EffectId, &VfsPathBuf)> {
        self.effects.iter().enumerate().filter_map(|(id, path)| {
            path.as_ref()
                .map(|path| (EffectId::new(id as u16).unwrap(), path))
        })
    }
}
