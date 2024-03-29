use log::{debug, warn};
use std::{collections::HashMap, sync::Arc};

use rose_data::{QuestData, QuestDatabase, StringDatabase, WorldTicks};
use rose_file_readers::{stb_column, QsdFile, StbFile, StbReadOptions, VirtualFilesystem};

struct StbQuest(StbFile);

impl StbQuest {
    stb_column! { 1, get_time_limit, WorldTicks }
}

pub fn get_quest_database(
    vfs: &VirtualFilesystem,
    string_database: Arc<StringDatabase>,
) -> Result<QuestDatabase, anyhow::Error> {
    let quest_s_stb = vfs.read_file_with::<StbFile, _>(
        "3DDATA/QUESTDATA/QUEST_S.STB",
        &StbReadOptions {
            is_wide: true,
            ..Default::default()
        },
    )?;
    let mut strings = HashMap::new();

    for row in 0..quest_s_stb.rows() {
        let english = quest_s_stb.get(row, 1);
        if !english.is_empty() {
            strings.insert(row as u16, english.to_string());
        }
    }

    let quest_stb = StbQuest(vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_QUEST.STB")?);
    let mut quests = Vec::new();
    for row in 0..quest_stb.0.rows() {
        let time_limit = quest_stb.get_time_limit(row).filter(|x| x.0 != 0);
        let string_id = quest_stb.0.try_get(row, quest_stb.0.columns() - 1);

        if let Some(string_id) = string_id {
            let quest_strings = string_database.get_quest(string_id);
            quests.push(Some(QuestData {
                id: row,
                name: quest_strings
                    .as_ref()
                    .map_or("", |x| unsafe { std::mem::transmute(x.name) }),
                description: quest_strings
                    .as_ref()
                    .map_or("", |x| unsafe { std::mem::transmute(x.description) }),
                start_message: quest_strings
                    .as_ref()
                    .map_or("", |x| unsafe { std::mem::transmute(x.start_message) }),
                end_message: quest_strings
                    .as_ref()
                    .map_or("", |x| unsafe { std::mem::transmute(x.end_message) }),
                time_limit,
            }));
        } else {
            quests.push(None);
        }
    }

    let qsd_files_stb = vfs.read_file::<StbFile, _>("3DDATA/STB/LIST_QUESTDATA.STB")?;
    let mut triggers = HashMap::new();

    for row in 0..qsd_files_stb.rows() {
        let qsd_path = qsd_files_stb.get(row, 0);
        if qsd_path.is_empty() {
            continue;
        }

        match vfs.read_file::<QsdFile, _>(qsd_path) {
            Ok(qsd) => triggers.extend(qsd.triggers),
            Err(error) => warn!("Failed to parse {}, error: {:?}", qsd_path, error),
        }
    }

    let mut triggers_by_hash = HashMap::new();
    for key in triggers.keys() {
        triggers_by_hash.insert(key.as_str().into(), key.clone());
    }

    debug!("Loaded {} QSD triggers", triggers.len());
    Ok(QuestDatabase {
        _string_database: string_database,
        quests,
        strings,
        triggers,
        triggers_by_hash,
    })
}
