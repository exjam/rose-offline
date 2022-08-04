use log::{debug, warn};
use rose_file_readers::{AipFile, StbFile, StbReadOptions, VirtualFilesystem};
use std::collections::HashMap;

use rose_data::AiDatabase;

pub fn get_ai_database(vfs: &VirtualFilesystem) -> Result<AiDatabase, anyhow::Error> {
    let ai_s_stb = vfs.read_file_with::<StbFile, _>(
        "3DDATA/AI/AI_S.STB",
        &StbReadOptions {
            is_wide: true,
            ..Default::default()
        },
    )?;
    let mut strings = HashMap::new();

    for row in 0..ai_s_stb.rows() {
        let english = ai_s_stb.get(row, 1);
        if !english.is_empty() {
            strings.insert(row as u16, english.to_string());
        }
    }

    let file_ai_stb = vfs.read_file::<StbFile, _>("3DDATA/STB/FILE_AI.STB")?;
    let mut aips = HashMap::new();

    for row in 0..file_ai_stb.rows() {
        let aip_path = file_ai_stb.get(row, 0);
        if aip_path.is_empty() {
            continue;
        }

        match vfs.read_file::<AipFile, _>(aip_path) {
            Ok(aip) => {
                aips.insert(row as u16, aip);
            }
            Err(error) => {
                warn!("Failed to parse {}, error: {:?}", aip_path, error);
            }
        }
    }

    debug!("Loaded {} AI files", aips.len());
    Ok(AiDatabase { strings, aips })
}
