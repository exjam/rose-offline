use serde::{Deserialize, Serialize};
use std::{str::FromStr, time::Duration};

use rose_file_readers::VfsPathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MotionId(u16);

id_wrapper_impl!(MotionId, u16);

#[derive(Clone, Default)]
pub struct MotionFileData {
    pub path: VfsPathBuf,
    pub duration: Duration,
    pub total_attack_frames: usize,
}
