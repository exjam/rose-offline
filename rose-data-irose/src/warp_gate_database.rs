use log::debug;
use std::collections::HashMap;

use rose_data::{WarpGateData, WarpGateDatabase, WarpGateId, ZoneId};
use rose_file_readers::{stb_column, StbFile, VirtualFilesystem};

pub struct StbWarp(pub StbFile);

#[allow(dead_code)]
impl StbWarp {
    pub fn rows(&self) -> usize {
        self.0.rows()
    }

    stb_column! { 0, get_warp_name, &str }
    stb_column! { 1, get_warp_target_zone, ZoneId }
    stb_column! { 2, get_warp_target_event_object, &str }
}

fn load_warp_gate(data: &StbWarp, id: usize) -> Option<WarpGateData> {
    Some(WarpGateData {
        target_zone: data.get_warp_target_zone(id)?,
        target_event_object: data.get_warp_target_event_object(id)?.to_string(),
    })
}

pub fn get_warp_gate_database(vfs: &VirtualFilesystem) -> Result<WarpGateDatabase, anyhow::Error> {
    let data = StbWarp(vfs.read_file::<StbFile, _>("3DDATA/STB/WARP.STB")?);
    let mut warp_gates = HashMap::new();
    for id in 1..data.rows() {
        if let Some(warp_gate_data) = load_warp_gate(&data, id) {
            warp_gates.insert(WarpGateId::new(id as u16), warp_gate_data);
        }
    }

    debug!("Loaded {} warp gates", warp_gates.len());
    Ok(WarpGateDatabase::new(warp_gates))
}
