use core::mem::size_of;
use std::{collections::HashMap, str};

use thiserror::Error;

use crate::{reader::FileReader, RoseFile};

pub struct StbFile {
    rows: usize,
    columns: usize,
    row_names: Vec<String>,
    _column_names: Vec<String>,
    data: Vec<u8>,
    cells: Vec<(usize, u16)>,
    row_keys: HashMap<String, usize>,
}

#[derive(Error, Debug)]
pub enum StbReadError {
    #[error("Invalid STB magic header")]
    InvalidMagic,
    #[error("Unsupported STB version")]
    UnsupportedVersion,
}

#[derive(Default)]
pub struct StbReadOptions {
    pub is_wide: bool,
    pub with_keys: bool,
}

impl RoseFile for StbFile {
    type ReadOptions = StbReadOptions;

    fn read(mut reader: FileReader, read_options: &StbReadOptions) -> Result<Self, anyhow::Error> {
        let magic = reader.read_fixed_length_string(3)?;
        if magic != "STB" {
            return Err(StbReadError::InvalidMagic.into());
        }

        if read_options.is_wide {
            reader.use_wide_strings = true;
        }

        StbFile::read_data(reader, read_options)
    }
}

#[allow(dead_code)]
impl StbFile {
    fn read_data(
        mut reader: FileReader,
        read_options: &StbReadOptions,
    ) -> Result<Self, anyhow::Error> {
        let version = {
            let version = reader.read_u8()?;
            if version == b'0' {
                0
            } else if version == b'1' {
                1
            } else {
                return Err(StbReadError::UnsupportedVersion.into());
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

        // Ignore the row / column headers
        let rows = row_count - 1;
        let columns = column_count - 1;

        reader.read_u16_length_string()?; // Ignore column title line

        let mut row_names = Vec::with_capacity(row_count);
        for _ in 0..rows {
            row_names.push(String::from(reader.read_u16_length_string()?));
        }

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

        let mut row_keys = HashMap::new();
        if read_options.with_keys {
            for (index, key) in row_names.iter().enumerate() {
                if !key.is_empty() {
                    row_keys.insert(key.clone(), index);
                }
            }
        }

        Ok(Self {
            rows,
            columns,
            row_names,
            _column_names: column_names,
            data,
            cells,
            row_keys,
        })
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
        self.try_get_row_name(row).unwrap_or("")
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
        self.try_get(row, column).unwrap_or("")
    }

    pub fn try_get_int(&self, row: usize, column: usize) -> Option<i32> {
        self.try_get(row, column)
            .and_then(|x| x.parse::<i32>().ok())
    }

    pub fn get_int(&self, row: usize, column: usize) -> i32 {
        self.try_get(row, column)
            .unwrap_or("")
            .parse::<i32>()
            .unwrap_or(0)
    }
}

#[macro_export]
macro_rules! stb_column {
    (
        $column_index:literal, $name:ident, &str
    ) => {
        pub fn $name(&self, row: usize) -> Option<&str> {
            self.0.try_get(row, $column_index)
        }
    };
    (
        $column_index:literal, $name:ident, bool
    ) => {
        pub fn $name(&self, row: usize) -> Option<bool> {
            self.0
                .try_get(row, $column_index)
                .and_then(|x| x.parse::<i32>().ok())
                .map(|x| x != 0)
        }
    };
    (
        $column_index:literal, $name:ident, $value_type:ty
    ) => {
        pub fn $name(&self, row: usize) -> Option<$value_type> {
            self.0
                .try_get(row, $column_index)
                .and_then(|x| x.parse::<$value_type>().ok())
        }
    };
    (
        $range:expr, $name:ident, ArrayVec< $value_type:ty, $len:literal >
    ) => {
        pub fn $name(&self, row: usize) -> ArrayVec<$value_type, $len> {
            let mut result: ArrayVec<$value_type, $len> = ArrayVec::new();

            for column in $range {
                if let Some(value) = self
                    .0
                    .try_get(row, column)
                    .and_then(|x| x.parse::<$value_type>().ok())
                {
                    result.push(value);
                }
            }

            result
        }
    };
    (
        $range:expr, $name:ident, [Option<$value_type:ty>; $len:literal]
    ) => {
        pub fn $name(&self, row: usize) -> [Option<$value_type>; $len] {
            let mut result: [Option<$value_type>; $len] = Default::default();

            for (i, column) in ($range).enumerate() {
                if let Some(value) = self
                    .0
                    .try_get(row, column)
                    .and_then(|x| x.parse::<$value_type>().ok())
                {
                    result[i] = Some(value);
                }
            }

            result
        }
    };
    (
        $range:expr, $name:ident, [$value_type:ty; $len:literal]
    ) => {
        pub fn $name(&self, row: usize) -> [$value_type; $len] {
            let mut result: [$value_type; $len] = Default::default();

            for (i, column) in ($range).enumerate() {
                result[i] = self
                    .0
                    .try_get(row, column)
                    .and_then(|x| x.parse::<$value_type>().ok())
                    .unwrap_or(0);
            }

            result
        }
    };
}
