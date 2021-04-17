use core::mem::size_of;
use encoding_rs::EUC_KR;
use std::borrow::Cow;
use std::str;

use super::reader::{FileReader, ReadError};

pub struct StbFile {
    rows: usize,
    columns: usize,
    data: Vec<u8>,
    cells: Vec<(usize, u16)>,
}

#[derive(Debug)]
pub enum STBReadError {
    InvalidMagic,
    UnsupportedVersion,
    UnexpectedEof,
}

impl From<ReadError> for STBReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => STBReadError::UnexpectedEof,
        }
    }
}

fn decode_string<'a>(mut bytes: &'a [u8]) -> Cow<'a, str> {
    for (length, c) in bytes.iter().enumerate() {
        if *c == 0 {
            bytes = &bytes[0..length];
            break;
        }
    }

    let (decoded, _, _) = EUC_KR.decode(bytes);
    decoded
}

#[allow(dead_code)]
impl StbFile {
    pub fn read(mut reader: FileReader) -> Result<Self, STBReadError> {
        let magic = reader.read_fixed_length_utf8(3)?;
        if magic != "STB" {
            return Err(STBReadError::InvalidMagic);
        }

        let version = {
            let version = reader.read_u8()?;
            if version == '0' as u8 {
                0
            } else if version == '1' as u8 {
                1
            } else {
                return Err(STBReadError::UnsupportedVersion);
            }
        };

        let data_position = reader.read_u32()? as u64;
        let row_count = reader.read_u32()? as usize;
        let column_count = reader.read_u32()? as usize;

        let _row_height = reader.read_u32();

        if version == 0 {
            let _column_width = reader.skip(size_of::<u32>() as u64);
        } else {
            let _column_widths = reader.skip((size_of::<u16>() * (column_count + 1)) as u64);
        }

        for _ in 0..column_count {
            let _column_name = reader.read_u16_length_bytes()?;
        }

        for _ in 0..row_count {
            let _row_name = reader.read_u16_length_bytes()?;
        }

        // Ignore the row / column headers
        let rows = row_count - 1;
        let columns = column_count - 1;

        let mut data = Vec::with_capacity(reader.remaining());
        let mut cells = Vec::with_capacity(row_count * column_count);

        reader.set_position(data_position);
        for _ in 0..rows {
            for _ in 0..columns {
                let cell = decode_string(reader.read_u16_length_bytes()?);
                let size = cell.as_bytes().len();
                let position = data.len();
                data.extend_from_slice(cell.as_bytes());
                cells.push((position, size as u16));
            }
        }

        Ok(Self {
            rows: rows,
            columns: columns,
            data: data,
            cells: cells,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn try_get(&self, row: usize, column: usize) -> Option<&str> {
        let cell_index = row * self.columns + column;
        if row >= self.rows || column >= self.columns || cell_index >= self.cells.len() {
            return None;
        }

        let (position, size) = self.cells[row * self.columns + column];
        if size == 0 {
            return None;
        }
        str::from_utf8(&self.data[position..(position + size as usize)]).ok()
    }

    pub fn get(&self, row: usize, column: usize) -> &str {
        self.try_get(row, column).unwrap_or(&"")
    }

    pub fn try_get_int(&self, row: usize, column: usize) -> Option<i32> {
        self.try_get(row, column)
            .and_then(|x| x.parse::<i32>().ok())
    }

    pub fn get_int(&self, row: usize, column: usize) -> i32 {
        self.try_get(row, column)
            .unwrap_or(&"")
            .parse::<i32>()
            .unwrap_or(0)
    }
}
