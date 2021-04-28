use core::mem::size_of;
use std::{collections::HashMap, str};

use super::reader::{FileReader, ReadError};

pub struct StbFile {
    rows: usize,
    columns: usize,
    row_names: Vec<String>,
    column_names: Vec<String>,
    data: Vec<u8>,
    cells: Vec<(usize, u16)>,
    row_keys: HashMap<String, usize>,
}

#[derive(Debug)]
pub enum StbReadError {
    InvalidMagic,
    UnsupportedVersion,
    UnexpectedEof,
}

impl From<ReadError> for StbReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => StbReadError::UnexpectedEof,
        }
    }
}

#[allow(dead_code)]
impl StbFile {
    pub fn read(mut reader: FileReader) -> Result<Self, StbReadError> {
        let magic = reader.read_fixed_length_string(3)?;
        if magic != "STB" {
            return Err(StbReadError::InvalidMagic);
        }

        let version = {
            let version = reader.read_u8()?;
            if version == '0' as u8 {
                0
            } else if version == '1' as u8 {
                1
            } else {
                return Err(StbReadError::UnsupportedVersion);
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

        let mut column_names = Vec::with_capacity(column_count);
        for _ in 0..column_count {
            column_names.push(String::from(reader.read_u16_length_string()?));
        }

        let mut row_names = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            row_names.push(String::from(reader.read_u16_length_string()?));
        }

        // Ignore the row / column headers
        let rows = row_count - 1;
        let columns = column_count - 1;

        let mut data = Vec::with_capacity(reader.remaining());
        let mut cells = Vec::with_capacity(row_count * column_count);

        reader.set_position(data_position);
        for _ in 0..rows {
            for _ in 0..columns {
                let cell = reader.read_u16_length_string()?;
                let size = cell.as_bytes().len();
                let position = data.len();
                data.extend_from_slice(cell.as_bytes());
                cells.push((position, size as u16));
            }
        }

        Ok(Self {
            rows: rows,
            columns: columns,
            row_names: row_names,
            column_names: column_names,
            data: data,
            cells: cells,
            row_keys: Default::default(),
        })
    }

    pub fn read_with_keys(reader: FileReader) -> Result<Self, StbReadError> {
        let mut stb = Self::read(reader)?;

        for (index, key) in stb.row_names.iter().enumerate() {
            if !key.is_empty() {
                stb.row_keys.insert(key.clone(), index);
            }
        }

        Ok(stb)
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn columns(&self) -> usize {
        self.columns
    }

    pub fn lookup_row_name(&self, name: &str) -> Option<usize> {
        self.row_keys.get(name).cloned()
    }

    pub fn try_get_row_name(&self, row: usize) -> Option<&str> {
        self.row_names.get(row).map(String::as_str)
    }

    pub fn get_row_name(&self, row: usize) -> &str {
        self.try_get_row_name(row).unwrap_or(&"")
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
