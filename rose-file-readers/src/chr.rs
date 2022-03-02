use std::collections::HashMap;

use crate::reader::{FileReader, ReadError};

pub struct NpcModelData {
    pub name: String,
    pub skeleton_index: u16,
    pub mesh_ids: Vec<u16>,
    pub motion_ids: Vec<(u16, u16)>, // (action, index)
    pub effect_ids: Vec<(u16, u16)>, // (action, index)
}

pub struct ChrFile {
    pub mesh_files: Vec<String>,
    pub motion_files: Vec<String>,
    pub effect_files: Vec<String>,
    pub npcs: HashMap<u16, NpcModelData>,
}

#[derive(Debug)]
pub enum ChrReadError {
    UnexpectedEof,
}

impl From<ReadError> for ChrReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => ChrReadError::UnexpectedEof,
        }
    }
}

#[allow(dead_code)]
impl ChrFile {
    pub fn read(mut reader: FileReader) -> Result<Self, ChrReadError> {
        let mesh_count = reader.read_u16()?;
        let mut mesh_files = Vec::new();
        for _ in 0..mesh_count {
            mesh_files.push(reader.read_null_terminated_string()?.to_string());
        }

        let motion_count = reader.read_u16()?;
        let mut motion_files = Vec::new();
        for _ in 0..motion_count {
            motion_files.push(reader.read_null_terminated_string()?.to_string());
        }

        let effect_count = reader.read_u16()?;
        let mut effect_files = Vec::new();
        for _ in 0..effect_count {
            effect_files.push(reader.read_null_terminated_string()?.to_string());
        }

        let character_count = reader.read_u16()?;
        let mut npcs = HashMap::new();
        for id in 0..character_count {
            if reader.read_u8()? == 0 {
                continue;
            }

            let skeleton_index = reader.read_u16()?;
            let name = reader.read_null_terminated_string()?.to_string();

            let mesh_count = reader.read_u16()?;
            let mut mesh_ids = Vec::new();
            for _ in 0..mesh_count {
                mesh_ids.push(reader.read_u16()?);
            }

            let motion_count = reader.read_u16()?;
            let mut motion_ids = Vec::new();
            for _ in 0..motion_count {
                let action = reader.read_u16()?;
                let motion_id = reader.read_u16()?;
                motion_ids.push((action, motion_id));
            }

            let effect_count = reader.read_u16()?;
            let mut effect_ids = Vec::new();
            for _ in 0..effect_count {
                let action = reader.read_u16()?;
                let effect_id = reader.read_u16()?;
                effect_ids.push((action, effect_id));
            }

            npcs.insert(
                id,
                NpcModelData {
                    name,
                    skeleton_index,
                    mesh_ids,
                    motion_ids,
                    effect_ids,
                },
            );
        }

        Ok(Self {
            mesh_files,
            motion_files,
            effect_files,
            npcs,
        })
    }
}
