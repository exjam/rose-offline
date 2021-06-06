use std::{collections::HashMap};

use crate::{
    data::{
        formats::{FileReader, StbFile, VfsIndex, ZmoFile},
        MotionCharacterAction, MotionDatabase, MotionFileData,
    },
};

fn load_zmo(vfs: &VfsIndex, path: &str) -> Option<MotionFileData> {
    let file = vfs.open_file(path)?;
    let zmo = ZmoFile::read(FileReader::from(&file)).ok()?;
    Some(MotionFileData {
        path: path.to_string(),
        duration: zmo.get_duration(),
        total_attack_frames: zmo.total_attack_frames,
    })
}

fn get_character_action(action_index: u16) -> Option<MotionCharacterAction> {
    match action_index {
        0 => Some(MotionCharacterAction::Stop1),
        1 => Some(MotionCharacterAction::Stop2),
        2 => Some(MotionCharacterAction::Walk),
        3 => Some(MotionCharacterAction::Run),
        4 => Some(MotionCharacterAction::Sitting),
        5 => Some(MotionCharacterAction::Sit),
        6 => Some(MotionCharacterAction::Standup),
        7 => Some(MotionCharacterAction::Stop3),
        8 => Some(MotionCharacterAction::Attack),
        9 => Some(MotionCharacterAction::Attack2),
        10 => Some(MotionCharacterAction::Attack3),
        11 => Some(MotionCharacterAction::Hit),
        12 => Some(MotionCharacterAction::Fall),
        13 => Some(MotionCharacterAction::Die),
        14 => Some(MotionCharacterAction::Raise),
        15 => Some(MotionCharacterAction::Jump1),
        16 => Some(MotionCharacterAction::Jump2),
        17 => Some(MotionCharacterAction::Pickitem),
        _ => None,
    }
}

pub fn get_motion_database(vfs: &VfsIndex) -> Option<MotionDatabase> {
    // Read motion file list
    let file = vfs.open_file("3DDATA/STB/FILE_MOTION.STB")?;
    let file_motion = StbFile::read(FileReader::from(&file)).ok()?;

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
    let file = vfs.open_file("3DDATA/STB/TYPE_MOTION.STB")?;
    let type_motion = StbFile::read(FileReader::from(&file)).ok()?;
    let num_character_motion_weapons = type_motion.columns();
    let num_character_motion_actions = type_motion.rows();
    let mut motion_indices = HashMap::new();

    for action in 0..num_character_motion_actions {
        if let Some(character_action) = get_character_action(action as u16) {
            let weapon_moticon_indices = motion_indices
                .entry(character_action)
                .or_insert_with(Vec::new);
            for weapon in 0..num_character_motion_weapons {
                weapon_moticon_indices.push(type_motion.get_int(action, weapon) as u16);
            }
        }
    }

    Some(MotionDatabase::new(motion_files, motion_indices))
}
