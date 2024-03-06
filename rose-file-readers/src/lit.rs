use anyhow::anyhow;

use crate::{reader::RoseFileReader, RoseFile};

#[derive(Debug)]
pub struct LitObject {
    pub id: u32,
    pub parts: Vec<LitObjectPart>,
}

#[derive(Debug)]
pub struct LitObjectPart {
    pub object_part_index: u32,
    pub filename: String,
    pub parts_per_row: u32,
    pub part_index: u32,
}

#[derive(Debug)]
pub struct LitFile {
    pub objects: Vec<LitObject>,
}

impl RoseFile for LitFile {
    type ReadOptions = ();
    type WriteOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let object_count = reader.read_u32()? as usize;
        if object_count > 10000 {
            return Err(anyhow!(
                "Corrupt .LIT file, invalid object_count {}",
                object_count
            ));
        }

        let mut objects = Vec::with_capacity(object_count);
        for _ in 0..object_count {
            let part_count = reader.read_u32()? as usize;
            if part_count > 10000 {
                return Err(anyhow!(
                    "Corrupt .LIT file, invalid part_count {}",
                    part_count
                ));
            }

            let object_id = reader.read_u32()?;
            let mut parts = Vec::with_capacity(part_count);
            for _ in 0..part_count {
                let name_len = reader.read_u8()?;
                reader.skip(name_len as u64);
                let object_part_index = reader.read_u32()?;
                let filename = reader.read_u8_length_string()?;
                reader.skip(8);
                let parts_per_row = reader.read_u32()?;
                let part_index = reader.read_u32()?;

                parts.push(LitObjectPart {
                    object_part_index,
                    filename: filename.to_string(),
                    parts_per_row,
                    part_index,
                });
            }

            objects.push(LitObject {
                id: object_id,
                parts,
            });
        }

        Ok(Self { objects })
    }
}
