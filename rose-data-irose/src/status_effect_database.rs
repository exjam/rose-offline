use arrayvec::ArrayVec;
use log::debug;
use std::{collections::HashMap, sync::Arc};

use rose_data::{
    EffectFileId, StatusEffectClearedByType, StatusEffectData, StatusEffectDatabase,
    StatusEffectId, StringDatabase,
};
use rose_file_readers::{stb_column, StbFile, VirtualFilesystem};

use crate::data_decoder::{IroseStatusEffectClearedByType, IroseStatusEffectType};

struct StbStatus(StbFile);

#[allow(dead_code)]
impl StbStatus {
    stb_column! { 1, get_status_effect_type, IroseStatusEffectType }
    stb_column! { 2, get_can_be_reapplied, bool }
    stb_column! { 3, get_cleared_by_type, IroseStatusEffectClearedByType }
    stb_column! { 4, get_apply_arg, i32 }
    stb_column! { (5..=8).step_by(2), get_apply_status_effect_id, [Option<StatusEffectId>; 2] }
    stb_column! { (5..=8).skip(1).step_by(2), get_apply_status_effect_value, [Option<i32>; 2] }

    pub fn get_apply_status_effects(&self, id: usize) -> ArrayVec<(StatusEffectId, i32), 2> {
        self.get_apply_status_effect_id(id)
            .iter()
            .zip(self.get_apply_status_effect_value(id).iter())
            .filter(|(a, b)| a.is_some() && b.is_some())
            .map(|(a, b)| (a.unwrap(), b.unwrap()))
            .collect()
    }

    stb_column! { 9, get_icon_id, u32 }
    stb_column! { 10, get_step_effect_file_id, EffectFileId }
    stb_column! { 11, get_step_sound_id, u32 }
    stb_column! { 12..=14, get_control, [Option<u32>; 3] }
    stb_column! { 15, get_end_effect_file_id, EffectFileId }
    stb_column! { 16, get_end_sound_id, u32 }
    stb_column! { 17, get_prifits_losses_by_state, i32 }
    stb_column! { 18, get_start_message_id, u32 }
    stb_column! { 19, get_end_message_id, u32 }
    stb_column! { 20, get_string_id, &str }
}

fn load_status_effect(
    data: &StbStatus,
    string_database: &StringDatabase,
    row: usize,
) -> Option<StatusEffectData> {
    let id = StatusEffectId::new(row as u16)?;
    let status_effect_type = data
        .get_status_effect_type(row)
        .and_then(|x| x.try_into().ok())?;

    let status_effect_strings = data
        .get_string_id(row)
        .and_then(|key| string_database.get_status_effect(key));

    Some(StatusEffectData {
        id,
        name: status_effect_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.name) }),
        description: status_effect_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.description) }),
        start_message: status_effect_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.start_message) }),
        end_message: status_effect_strings
            .as_ref()
            .map_or("", |x| unsafe { std::mem::transmute(x.end_message) }),
        status_effect_type,
        can_be_reapplied: data.get_can_be_reapplied(row).unwrap_or(false),
        cleared_by_type: data
            .get_cleared_by_type(row)
            .and_then(|x| x.try_into().ok())
            .unwrap_or(StatusEffectClearedByType::ClearGood),
        apply_status_effects: data.get_apply_status_effects(row),
        apply_per_second_value: data.get_apply_status_effect_value(row)[0].unwrap_or(0),
        effect_file_id: data.get_step_effect_file_id(row),
        icon_id: data.get_icon_id(row).unwrap_or(0),
    })
}

pub fn get_status_effect_database(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
) -> Result<StatusEffectDatabase, anyhow::Error> {
    let data = StbStatus(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_STATUS.STB")?);
    let mut status_effects = HashMap::new();

    for row in 1..data.0.rows() {
        if let Some(status_effect_data) = load_status_effect(&data, &string_database, row) {
            status_effects.insert(row as u16, status_effect_data);
        }
    }

    debug!("Loaded {} status effects", status_effects.len());
    Ok(StatusEffectDatabase::new(
        string_database,
        status_effects,
        StatusEffectId::new(43).unwrap(),
    ))
}
