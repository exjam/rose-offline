use std::collections::HashMap;

use rose_data::{MotionDatabase, MotionFileData};
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