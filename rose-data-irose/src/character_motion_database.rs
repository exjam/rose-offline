use enum_map::{enum_map, EnumMap};
use rose_data::{
    CharacterMotionAction, CharacterMotionDatabase, CharacterMotionDatabaseOptions, MotionFileData,
    MotionId,
};
use rose_file_readers::{StbFile, VfsIndex, ZmoFile};

fn get_action_map() -> EnumMap<CharacterMotionAction, MotionId> {
    enum_map! {
        CharacterMotionAction::Stop1 => MotionId::new(0),
        CharacterMotionAction::Stop2 => MotionId::new(1),
        CharacterMotionAction::Walk => MotionId::new(2),
        CharacterMotionAction::Run => MotionId::new(3),
        CharacterMotionAction::Sitting => MotionId::new(4),
        CharacterMotionAction::Sit => MotionId::new(5),
        CharacterMotionAction::Standup => MotionId::new(6),
        CharacterMotionAction::Stop3 => MotionId::new(7),
        CharacterMotionAction::Attack => MotionId::new(8),
        CharacterMotionAction::Attack2 => MotionId::new(9),
        CharacterMotionAction::Attack3 => MotionId::new(10),
        CharacterMotionAction::Hit => MotionId::new(11),
        CharacterMotionAction::Fall => MotionId::new(12),
        CharacterMotionAction::Die => MotionId::new(13),
        CharacterMotionAction::Raise => MotionId::new(14),
        CharacterMotionAction::Jump1 => MotionId::new(15),
        CharacterMotionAction::Jump2 => MotionId::new(16),
        CharacterMotionAction::Pickitem => MotionId::new(17),
    }
}

fn load_motion_file_data(
    vfs: &VfsIndex,
    path: &str,
    options: &CharacterMotionDatabaseOptions,
) -> Option<MotionFileData> {
    if path.is_empty() {
        return None;
    }

    if options.load_frame_data {
        let zmo = vfs.read_file::<ZmoFile, _>(path).ok()?;
        Some(MotionFileData {
            path: path.to_string(),
            duration: zmo.get_duration(),
            total_attack_frames: zmo.total_attack_frames,
        })
    } else {
        Some(MotionFileData {
            path: path.to_string(),
            ..Default::default()
        })
    }
}

pub fn get_character_motion_database(
    vfs: &VfsIndex,
    options: &CharacterMotionDatabaseOptions,
) -> Option<CharacterMotionDatabase> {
    // Read motion file list
    let file_motion = vfs
        .read_file::<StbFile, _>("3DDATA/STB/FILE_MOTION.STB")
        .ok()?;

    let mut motion_datas = Vec::new();
    for gender in 0..file_motion.columns() {
        let mut gender_motions = Vec::new();
        for index in 0..file_motion.rows() {
            gender_motions.push(load_motion_file_data(
                vfs,
                file_motion.get(index, gender),
                options,
            ));
        }
        motion_datas.push(gender_motions);
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

    Some(CharacterMotionDatabase::new(
        num_character_motion_weapons,
        motion_indices,
        motion_datas,
        get_action_map(),
    ))
}
