use std::{collections::HashMap, io::BufRead, ops::Deref};

use crate::{reader::RoseFileReader, RoseFile};

pub struct IdFile(HashMap<String, i32>);

impl Deref for IdFile {
    type Target = HashMap<String, i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RoseFile for IdFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let mut line = String::new();
        let mut ids = HashMap::new();

        while reader.cursor.read_line(&mut line).is_ok() {
            if line.is_empty() {
                break;
            }

            let mut split_itr = line.split_ascii_whitespace();
            let key = if let Some(key) = split_itr.next() {
                key
            } else {
                continue;
            };

            if key.is_empty() {
                continue;
            }

            let value =
                if let Some(value) = split_itr.next().and_then(|value| value.parse::<i32>().ok()) {
                    value
                } else {
                    continue;
                };

            ids.insert(key.to_string(), value);
            line.clear();
        }

        Ok(Self(ids))
    }
}
