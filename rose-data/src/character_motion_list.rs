use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct CharacterMotionId(u16);

id_wrapper_impl!(CharacterMotionId, u16);

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

pub struct CharacterMotionList {
    weapon_type_count: usize,
    motion_indices: Vec<u16>,
    motion_paths: Vec<Vec<String>>, // [gender][motion id]
    action_map: EnumMap<CharacterMotionAction, CharacterMotionId>,
}

impl CharacterMotionList {
    pub fn new(
        weapon_type_count: usize,
        motion_indices: Vec<u16>,
        motion_paths: Vec<Vec<String>>,
        action_map: EnumMap<CharacterMotionAction, CharacterMotionId>,
    ) -> Self {
        Self {
            weapon_type_count,
            motion_indices,
            motion_paths,
            action_map,
        }
    }

    pub fn get_character_motion(
        &self,
        motion_id: CharacterMotionId,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&str> {
        let index = *self
            .motion_indices
            .get(motion_id.get() as usize * self.weapon_type_count + weapon_motion_type)?
            as usize;

        self.motion_paths
            .get(gender)
            .and_then(|x| x.get(index).filter(|x| !x.is_empty()).map(|x| x.as_str()))
    }

    pub fn find_first_character_motion(&self, motion_id: CharacterMotionId) -> Option<&str> {
        // Try find the first non-empty motion for every weapon_type & gender for an action
        for gender in 0..self.motion_paths.len() {
            for weapon_motion_type in 0..self.weapon_type_count {
                if let Some(path) = self.get_character_motion(motion_id, weapon_motion_type, gender)
                {
                    return Some(path);
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
    ) -> Option<&str> {
        self.get_character_motion(self.action_map[action], weapon_motion_type, gender)
    }

    pub fn find_first_character_action_motion(
        &self,
        action: CharacterMotionAction,
    ) -> Option<&str> {
        self.find_first_character_motion(self.action_map[action])
    }
}
