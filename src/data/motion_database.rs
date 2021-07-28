use std::{collections::HashMap, num::NonZeroUsize, time::Duration};

use crate::game::components::{MotionData, MotionDataCharacter};

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

pub struct MotionReference(NonZeroUsize);

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
        motion: MotionReference,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        let index = self
            .motion_indices
            .get(motion.0.get() * self.weapoon_type_count + weapon_motion_type)?;

        self.motion_files.get(gender).and_then(|x| x.get(index))
    }

    #[allow(dead_code)]
    pub fn find_first_character_motion(&self, motion: MotionReference) -> Option<&MotionFileData> {
        let motion = motion.0.get();

        // Try find the first set motion for every weapon_type & gender for an action index
        for gender in 0..self.motion_files.len() {
            for i in 0..self.weapoon_type_count {
                if let Some(index) = self
                    .motion_indices
                    .get(i + motion * self.weapoon_type_count)
                {
                    if let Some(data) = self.motion_files.get(gender).and_then(|x| x.get(index)) {
                        return Some(data);
                    }
                }
            }
        }

        None
    }

    pub fn get_character_action_motions(
        &self,
        weapon_motion_type: usize,
        gender: usize,
    ) -> MotionData {
        let get_motion = |action| {
            self.get_character_motion(
                MotionReference(NonZeroUsize::new(action as usize).unwrap()),
                weapon_motion_type,
                gender,
            )
            .cloned()
        };

        MotionData::with_character_motions(MotionDataCharacter {
            weapon_motion_type,
            gender,
            attack1: get_motion(MotionCharacterAction::Attack),
            attack2: get_motion(MotionCharacterAction::Attack2),
            attack3: get_motion(MotionCharacterAction::Attack3),
            die: get_motion(MotionCharacterAction::Die),
            fall: get_motion(MotionCharacterAction::Fall),
            hit: get_motion(MotionCharacterAction::Hit),
            jump1: get_motion(MotionCharacterAction::Jump1),
            jump2: get_motion(MotionCharacterAction::Jump2),
            pickup_dropped_item: get_motion(MotionCharacterAction::Pickitem),
            raise: get_motion(MotionCharacterAction::Raise),
            run: get_motion(MotionCharacterAction::Run),
            sit: get_motion(MotionCharacterAction::Sit),
            sitting: get_motion(MotionCharacterAction::Sitting),
            standup: get_motion(MotionCharacterAction::Standup),
            stop1: get_motion(MotionCharacterAction::Stop1),
            stop2: get_motion(MotionCharacterAction::Stop2),
            stop3: get_motion(MotionCharacterAction::Stop3),
            walk: get_motion(MotionCharacterAction::Walk),
        })
    }
}
