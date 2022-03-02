use log::{debug, warn};
use rose_file_readers::{AipFile, FileReader, StbFile, VfsIndex};
use std::collections::HashMap;

use crate::data::AiDatabase;

pub fn get_ai_database(vfs: &VfsIndex) -> Option<AiDatabase> {
    let ai_s = vfs.open_file("3DDATA/AI/AI_s.STB")?;
    let ai_s_stb = StbFile::read_wide(FileReader::from(&ai_s)).ok()?;
    let mut strings = HashMap::new();

    for row in 0..ai_s_stb.rows() {
        let english = ai_s_stb.get(row, 1);
        if !english.is_empty() {
            strings.insert(row as u16, english.to_string());
        }
    }

    let file_ai = vfs.open_file("3DDATA/STB/FILE_AI.STB")?;
    let file_ai_stb = StbFile::read(FileReader::from(&file_ai)).ok()?;
    let mut aips = HashMap::new();

    for row in 0..file_ai_stb.rows() {
        let aip_path = file_ai_stb.get(row, 0);
        if aip_path.is_empty() {
            continue;
        }

        if let Some(aip_file) = vfs.open_file(aip_path) {
            match AipFile::read(FileReader::from(&aip_file)) {
                Ok(aip) => {
                    aips.insert(row as u16, aip);
                }
                Err(error) => {
                    warn!("Failed to parse {}, error: {:?}", aip_path, error);
                }
            }
        }
    }

    debug!("Loaded {} AI files", aips.len());
    Some(AiDatabase { strings, aips })
}
