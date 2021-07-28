use std::collections::HashMap;

use crate::data::{
    formats::{FileReader, StbFile, VfsIndex, ZmoFile},
    MotionDatabase, MotionFileData,
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
