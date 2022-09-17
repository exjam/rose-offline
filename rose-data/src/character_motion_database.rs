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

#[derive(Copy, Clone, Debug, Enum)]
pub enum VehicleMotionAction {
    Stop,
    Move,
    Attack1,
    Attack2,
    Attack3,
    Die,
    Special1,
    Special2,
}

pub struct CharacterMotionDatabase {
    weapon_type_count: usize,
    motion_indices: Vec<u16>,
    motion_data: Vec<Vec<Option<MotionFileData>>>, // [gender][motion id]
    action_map: EnumMap<CharacterMotionAction, MotionId>,
    vehicle_action_map: EnumMap<VehicleMotionAction, u16>,
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
        vehicle_action_map: EnumMap<VehicleMotionAction, u16>,
    ) -> Self {
        Self {
            weapon_type_count,
            motion_indices,
            motion_data: motion_paths,
            action_map,
            vehicle_action_map,
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

    pub fn find_first_character_motion(
        &self,
        motion_id: MotionId,
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        // Check if weapon has a motion index
        let mut index = *self
            .motion_indices
            .get(motion_id.get() as usize * self.weapon_type_count + weapon_motion_type)?
            as usize;

        // Fallback to no weapon
        if index == 0 {
            index = *self
                .motion_indices
                .get(motion_id.get() as usize * self.weapon_type_count)?
                as usize;
        }

        // Check if gender != 0 has motion, else fall back to gender 0
        if gender != 0 {
            if let Some(motion_file_data) = self
                .motion_data
                .get(gender)
                .and_then(|x| x.get(index).and_then(|x| x.as_ref()))
            {
                return Some(motion_file_data);
            }
        }

        self.motion_data
            .get(0)
            .and_then(|x| x.get(index).and_then(|x| x.as_ref()))
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
        weapon_motion_type: usize,
        gender: usize,
    ) -> Option<&MotionFileData> {
        self.find_first_character_motion(self.action_map[action], weapon_motion_type, gender)
    }

    pub fn get_vehicle_action_motion(
        &self,
        action: VehicleMotionAction,
        base_motion_index: usize,
    ) -> Option<&MotionFileData> {
        let index = *self.motion_indices.get(
            base_motion_index * self.weapon_type_count + self.vehicle_action_map[action] as usize,
        )? as usize;

        self.motion_data
            .get(0)
            .and_then(|x| x.get(index).and_then(|x| x.as_ref()))
    }
}
