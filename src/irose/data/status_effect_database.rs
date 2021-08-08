use std::{collections::HashMap, str::FromStr};

use arrayvec::ArrayVec;
use log::debug;
use num_traits::FromPrimitive;

use crate::{
    data::{
        formats::{FileReader, StbFile, StlFile, VfsIndex},
        StatusEffectClearedByType, StatusEffectData, StatusEffectDatabase, StatusEffectId,
        StatusEffectType,
    },
    stb_column,
};

impl FromStr for StatusEffectClearedByType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

impl FromStr for StatusEffectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u32>().map_err(|_| ())?;
        FromPrimitive::from_u32(value).ok_or(())
    }
}

struct StbStatus(StbFile);

#[allow(dead_code)]
impl StbStatus {
    stb_column! { 1, get_status_effect_type, StatusEffectType }
    stb_column! { 2, get_can_be_reapplied, bool }
    stb_column! { 3, get_cleared_by_type, StatusEffectClearedByType }
    stb_column! { 4, get_apply_arg, i32 }
    stb_column! { (5..=8).step_by(2), get_apply_status_effect_id, [Option<StatusEffectId>; 2] }
    stb_column! { (5..=8).step_by(2), get_apply_status_effect_value, [Option<i32>; 2] }

    pub fn get_apply_status_effects(&self, id: usize) -> ArrayVec<(StatusEffectId, i32), 2> {
        self.get_apply_status_effect_id(id)
            .iter()
            .zip(self.get_apply_status_effect_value(id).iter())
            .filter(|(a, b)| a.is_some() && b.is_some())
            .map(|(a, b)| (a.unwrap(), b.unwrap()))
            .collect()
    }

    stb_column! { 9, get_symbol_id, u32 }
    stb_column! { 10, get_step_effect_id, u32 }
    stb_column! { 11, get_step_sound_id, u32 }
    stb_column! { 12..=14, get_control, [Option<u32>; 3] }
    stb_column! { 15, get_end_effect_id, u32 }
    stb_column! { 16, get_end_sound_id, u32 }
    stb_column! { 17, get_prifits_losses_by_state, i32 }
    stb_column! { 18, get_start_message_id, u32 }
    stb_column! { 19, get_end_message_id, u32 }
    stb_column! { 20, get_string_id, &str }
}

fn load_status_effect(data: &StbStatus, stl: &StlFile, row: usize) -> Option<StatusEffectData> {
    let id = StatusEffectId::new(row as u16)?;
    let status_effect_type = data.get_status_effect_type(row)?;

    Some(StatusEffectData {
        id,
        name: data
            .get_string_id(row)
            .and_then(|string_id| stl.get_text_string(1, string_id))
            .unwrap_or("")
            .to_string(),
        status_effect_type,
        can_be_reapplied: data.get_can_be_reapplied(row).unwrap_or(false),
        cleared_by_type: data
            .get_cleared_by_type(row)
            .unwrap_or(StatusEffectClearedByType::ClearGood),
        apply_status_effects: data.get_apply_status_effects(row),
    })
}

pub fn get_status_effect_database(vfs: &VfsIndex) -> Option<StatusEffectDatabase> {
    let file = vfs.open_file("3DDATA/STB/LIST_STATUS_S.STL")?;
    let stl = StlFile::read(FileReader::from(&file)).ok()?;

    let file = vfs.open_file("3DDATA/STB/LIST_STATUS.STB")?;
    let data = StbStatus(StbFile::read(FileReader::from(&file)).ok()?);
    let mut status_effects = HashMap::new();

    for row in 1..data.0.rows() {
        if let Some(status_effect_data) = load_status_effect(&data, &stl, row) {
            status_effects.insert(row as u16, status_effect_data);
        }
    }

    debug!("Loaded {} status effects", status_effects.len());
    Some(StatusEffectDatabase::new(status_effects))
}
