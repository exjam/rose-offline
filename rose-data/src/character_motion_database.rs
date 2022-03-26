use enum_map::{Enum, EnumMap};

use crate::{MotionFileData, MotionId};

#[derive(Copy, Clone, Debug, Enum)]
pub enum CharacterMotionAction {
    Stop1,
    Stop2,
    Walk,
    Run,
    Sitting,
    Sit,
    Standup,
    Stop3,
    Attack,
    Attack2,
    Attack3,
    Hit,
    Fall,
    Die,
    Raise,
    Jump1,
    Jump2,
    Pickitem,
}

pub struct CharacterMotionDatabase {
    weapon_type_count: usize,
    motion_indices: Vec<u16>,
    motion_data: Vec<Vec<Option<MotionFileData>>>, // [gender][motion id]
    action_map: EnumMap<CharacterMotionAction, MotionId>,
}

pub struct CharacterMotionDatabaseOptions {
    pub load_frame_data: bool,
}

impl CharacterMotionDatabase {
    pub fn new(
        weapon_type_count: usize,
        motion_indices: Vec<u16>,
        motion_paths: Vec<Vec<Option<MotionFileData>>>,
        action_map: EnumMap<CharacterMotionAction, MotionId>,
    ) -> Self {
        Self {
            weapon_type_count,
            motion_indices,
            motion_data: motion_paths,
            action_map,
        }
    }

    pub fn get_character_motion(
        &self,
        motion_id: MotionId,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        let index = *self
            .motion_indices
            .get(motion_id.get() as usize * self.weapon_type_count + weapon_motion_type)?
            as usize;

        self.motion_data
            .get(gender)
            .and_then(|x| x.get(index).and_then(|x| x.as_ref()))
    }

    pub fn find_first_character_motion(&self, motion_id: MotionId) -> Option<&MotionFileData> {
        // Try find the first non-empty motion for every weapon_type & gender for an action
        for gender in 0..self.motion_data.len() {
            for weapon_motion_type in 0..self.weapon_type_count {
                if let Some(data) = self.get_character_motion(motion_id, weapon_motion_type, gender)
                {
                    return Some(data);
                }
            }
        }

        None
    }

    pub fn get_character_action_motion(
        &self,
        action: CharacterMotionAction,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        self.get_character_motion(self.action_map[action], weapon_motion_type, gender)
    }

    pub fn find_first_character_action_motion(
        &self,
        action: CharacterMotionAction,
    ) -> Option<&MotionFileData> {
        self.find_first_character_motion(self.action_map[action])
    }
}
