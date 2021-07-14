use std::{collections::HashMap, time::Duration};

use crate::{
    data::{
        formats::{qsd::QsdFile, FileReader, StbFile, VfsIndex},
        QuestData, QuestDatabase,
    },
    stb_column,
};

struct StbQuest(StbFile);

impl StbQuest {
    stb_column! { 1, get_time_limit, u64 }
}

pub fn get_quest_database(vfs: &VfsIndex) -> Option<QuestDatabase> {
    let file = vfs.open_file("3DDATA/QUESTDATA/QUEST_S.STB")?;
    let quest_s_stb = StbFile::read_wide(FileReader::from(&file)).ok()?;
    let mut strings = HashMap::new();

    for row in 0..quest_s_stb.rows() {
        let english = quest_s_stb.get(row, 1);
        if !english.is_empty() {
            strings.insert(row as u16, english.to_string());
        }
    }

    let file = vfs.open_file("3DDATA/STB/LIST_QUEST.STB")?;
    let quest_stb = StbQuest(StbFile::read(FileReader::from(&file)).ok()?);
    let mut quests = Vec::new();
    for row in 0..quest_stb.0.rows() {
        let time_limit = quest_stb
            .get_time_limit(row)
            .filter(|v| *v != 0)
            .map(Duration::from_secs);
        quests.push(QuestData { time_limit });
    }

    let file = vfs.open_file("3DDATA/STB/LIST_QUESTDATA.STB")?;
    let qsd_files_stb = StbFile::read(FileReader::from(&file)).ok()?;
    let mut triggers = HashMap::new();

    for row in 0..qsd_files_stb.rows() {
        let qsd_path = qsd_files_stb.get(row, 0);
        if qsd_path.is_empty() {
            continue;
        }

        if let Some(qsd_file) = vfs.open_file(qsd_path) {
            match QsdFile::read(FileReader::from(&qsd_file)) {
                Ok(qsd) => triggers.extend(qsd.triggers),
                Err(error) => println!("Failed to parse {}, error: {:?}", qsd_path, error),
            }
        }
    }

    let mut triggers_by_hash = HashMap::new();
    for key in triggers.keys() {
        triggers_by_hash.insert(key.as_str().into(), key.clone());
    }

    Some(QuestDatabase {
        quests,
        strings,
        triggers,
        triggers_by_hash,
    })
}
