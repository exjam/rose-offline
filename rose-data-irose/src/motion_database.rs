use std::collections::HashMap;

use enum_map::enum_map;
use rose_data::{
    CharacterMotionAction, CharacterMotionId, CharacterMotionList, MotionDatabase, MotionFileData,
};
use rose_file_readers::{StbFile, VfsIndex, ZmoFile};

fn load_zmo(vfs: &VfsIndex, path: &str) -> Option<MotionFileData> {
    let zmo = vfs.read_file::<ZmoFile, _>(path).ok()?;
    Some(MotionFileData {
        path: path.to_string(),
        duration: zmo.get_duration(),
        total_attack_frames: zmo.total_attack_frames,
    })
}

pub fn get_motion_database(vfs: &VfsIndex) -> Option<MotionDatabase> {
    // Read motion file list
    let file_motion = vfs
        .read_file::<StbFile, _>("3DDATA/STB/FILE_MOTION.STB")
        .ok()?;

    let mut motion_files = Vec::new();
    for gender in 0..file_motion.columns() {
        let mut gender_motions = HashMap::new();
        for index in 0..file_motion.rows() {
            let path = file_motion.get(index, gender);
            if !path.is_empty() {
                if let Some(motion_data) = load_zmo(vfs, path) {
                    gender_motions.insert(index as u16, motion_data);
                }
            }
        }
        motion_files.push(gender_motions);
    }

    // Read character motion mappings
    let type_motion = vfs
        .read_file::<StbFile, _>("3DDATA/STB/TYPE_MOTION.STB")
        .ok()?;
    let num_character_motion_weapons = type_motion.columns();
    let num_character_motion_actions = type_motion.rows();
    let mut motion_indices = Vec::new();

    for action in 0..num_character_motion_actions {
        for weapon in 0..num_character_motion_weapons {
            motion_indices.push(type_motion.get_int(action, weapon) as u16);
        }
    }

    Some(MotionDatabase::new(
        num_character_motion_weapons,
        motion_indices,
        motion_files,
    ))
}

pub fn get_character_motion_list(vfs: &VfsIndex) -> Option<CharacterMotionList> {
    // Read motion file list
    let file_motion = vfs
        .read_file::<StbFile, _>("3DDATA/STB/FILE_MOTION.STB")
        .ok()?;

    let mut motion_paths = Vec::new();
    for gender in 0..file_motion.columns() {
        let mut gender_motions = Vec::with_capacity(file_motion.rows());
        for index in 0..file_motion.rows() {
            gender_motions.push(file_motion.get(index, gender).to_string());
        }
        motion_paths.push(gender_motions);
    }

    // Read character motion mappings
    let type_motion = vfs
        .read_file::<StbFile, _>("3DDATA/STB/TYPE_MOTION.STB")
        .ok()?;
    let num_character_motion_weapons = type_motion.columns();
    let num_character_motion_actions = type_motion.rows();
    let mut motion_indices = Vec::new();

    for action in 0..num_character_motion_actions {
        for weapon in 0..num_character_motion_weapons {
            motion_indices.push(type_motion.get_int(action, weapon) as u16);
        }
    }

    let action_map = enum_map! {
        CharacterMotionAction::Stop1 => CharacterMotionId::new(0),
        CharacterMotionAction::Stop2 => CharacterMotionId::new(1),
        CharacterMotionAction::Walk => CharacterMotionId::new(2),
        CharacterMotionAction::Run => CharacterMotionId::new(3),
        CharacterMotionAction::Sitting => CharacterMotionId::new(4),
        CharacterMotionAction::Sit => CharacterMotionId::new(5),
        CharacterMotionAction::Standup => CharacterMotionId::new(6),
        CharacterMotionAction::Stop3 => CharacterMotionId::new(7),
        CharacterMotionAction::Attack => CharacterMotionId::new(8),
        CharacterMotionAction::Attack2 => CharacterMotionId::new(9),
        CharacterMotionAction::Attack3 => CharacterMotionId::new(10),
        CharacterMotionAction::Hit => CharacterMotionId::new(11),
        CharacterMotionAction::Fall => CharacterMotionId::new(12),
        CharacterMotionAction::Die => CharacterMotionId::new(13),
        CharacterMotionAction::Raise => CharacterMotionId::new(14),
        CharacterMotionAction::Jump1 => CharacterMotionId::new(15),
        CharacterMotionAction::Jump2 => CharacterMotionId::new(16),
        CharacterMotionAction::Pickitem => CharacterMotionId::new(17),
    };

    Some(CharacterMotionList::new(
        num_character_motion_weapons,
        motion_indices,
        motion_paths,
        action_map,
    ))
}
