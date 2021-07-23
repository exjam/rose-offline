use std::{collections::HashMap, time::Duration};

use crate::game::components::MotionData;

#[derive(Clone)]
pub struct MotionFileData {
    pub path: String,
    pub duration: Duration,
    pub total_attack_frames: usize,
}

#[derive(PartialEq, Eq, Hash)]
pub enum MotionCharacterAction {
    Attack,
    Attack2,
    Attack3,
    Die,
    Fall,
    Hit,
    Jump1,
    Jump2,
    Pickitem,
    Raise,
    Run,
    Sit,
    Sitting,
    Standup,
    Stop1,
    Stop2,
    Stop3,
    Walk,
}

pub struct MotionDatabase {
    motion_files: Vec<HashMap<u16, MotionFileData>>,
    motion_indices: HashMap<MotionCharacterAction, Vec<u16>>,
}

impl MotionDatabase {
    pub fn new(
        motion_files: Vec<HashMap<u16, MotionFileData>>,
        motion_indices: HashMap<MotionCharacterAction, Vec<u16>>,
    ) -> Self {
        Self {
            motion_files,
            motion_indices,
        }
    }

    pub fn get_character_motion(
        &self,
        action: MotionCharacterAction,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        let index = self
            .motion_indices
            .get(&action)
            .and_then(|x| x.get(weapon_motion_type))?;

        self.motion_files.get(gender).and_then(|x| x.get(index))
    }

    pub fn get_character_motions(&self, weapon_motion_type: usize, gender: usize) -> MotionData {
        MotionData {
            attack: self
                .get_character_motion(MotionCharacterAction::Attack, weapon_motion_type, gender)
                .cloned(),
            die: self
                .get_character_motion(MotionCharacterAction::Die, weapon_motion_type, gender)
                .cloned(),
            pickup_dropped_item: self
                .get_character_motion(MotionCharacterAction::Pickitem, weapon_motion_type, gender)
                .cloned(),
        }
    }
}
