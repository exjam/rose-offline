use crate::{reader::RoseFileReader, RoseFile};

#[derive(Default)]
pub struct LtbFile {
    pub rows: usize,
    pub columns: usize,
    pub cells: Vec<(u32, u16)>,
    pub data_offset: u32,
    pub data: Vec<u16>,
}

impl LtbFile {
    pub fn get_string(&self, row: usize, column: usize) -> Option<String> {
        let &(offset, size) = self.cells.get(row * self.columns + column)?;
        if offset < self.data_offset || size == 0 {
            None
        } else {
            let offset = (offset - self.data_offset) as usize / 2;
            Some(String::from_utf16_lossy(
                &self.data[offset..offset + size as usize],
            ))
        }
    }
}

impl RoseFile for LtbFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &()) -> Result<Self, anyhow::Error> {
        let columns = reader.read_u32()? as usize;
        let rows = reader.read_u32()? as usize;

        let mut cells = Vec::with_capacity(rows * columns);
        for _ in 0..rows {
            for _ in 0..columns {
                let position = reader.read_u32()?;
                let size = reader.read_u16()?;
                cells.push((position, size as u16));
            }
        }

        let data_offset = reader.position() as u32;
        let data = reader.read_vec(reader.remaining() / 2)?;

        Ok(Self {
            rows,
            columns,
            cells,
            data_offset,
            data,
        })
    }
}
