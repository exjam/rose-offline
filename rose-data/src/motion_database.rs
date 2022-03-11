use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, time::Duration};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct MotionId(u16);

id_wrapper_impl!(MotionId, u16);

#[derive(Clone)]
pub struct MotionFileData {
    pub path: String,
    pub duration: Duration,
    pub total_attack_frames: usize,
}

pub enum MotionCharacterAction {
    Stop1 = 0,
    Stop2 = 1,
    Walk = 2,
    Run = 3,
    Sitting = 4,
    Sit = 5,
    Standup = 6,
    Stop3 = 7,
    Attack = 8,
    Attack2 = 9,
    Attack3 = 10,
    Hit = 11,
    Fall = 12,
    Die = 13,
    Raise = 14,
    Jump1 = 15,
    Jump2 = 16,
    Pickitem = 17,
}

pub struct MotionDatabase {
    weapoon_type_count: usize,
    motion_indices: Vec<u16>,
    motion_files: Vec<HashMap<u16, MotionFileData>>,
}

impl MotionDatabase {
    pub fn new(
        weapoon_type_count: usize,
        motion_indices: Vec<u16>,
        motion_files: Vec<HashMap<u16, MotionFileData>>,
    ) -> Self {
        Self {
            weapoon_type_count,
            motion_indices,
            motion_files,
        }
    }

    pub fn get_character_motion(
        &self,
        motion_id: MotionId,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        let index = self
            .motion_indices
            .get(motion_id.get() as usize * self.weapoon_type_count + weapon_motion_type)?;

        self.motion_files.get(gender).and_then(|x| x.get(index))
    }

    pub fn find_first_character_motion(&self, motion_id: MotionId) -> Option<&MotionFileData> {
        let motion_id = motion_id.get() as usize;

        // Try find the first set motion for every weapon_type & gender for an action index
        for gender in 0..self.motion_files.len() {
            for i in 0..self.weapoon_type_count {
                if let Some(index) = self
                    .motion_indices
                    .get(i + motion_id * self.weapoon_type_count)
                {
                    if let Some(data) = self.motion_files.get(gender).and_then(|x| x.get(index)) {
                        return Some(data);
                    }
                }
            }
        }

        None
    }
}
