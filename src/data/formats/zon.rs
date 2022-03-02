use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::data::formats::reader::{FileReader, ReadError};

pub struct ZonFile {
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub event_positions: Vec<(String, [f32; 3])>,
}

#[derive(Debug)]
pub enum ZonReadError {
    UnexpectedEof,
}

impl From<ReadError> for ZonReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => ZonReadError::UnexpectedEof,
        }
    }
}

#[derive(FromPrimitive)]
enum BlockType {
    ZoneInfo = 0,
    EventPositions = 1,
    Textures = 2,
    Tiles = 3,
    Economy = 4,
}

#[allow(dead_code)]
impl ZonFile {
    pub fn read(mut reader: FileReader) -> Result<Self, ZonReadError> {
        let mut event_positions = Vec::new();
        let mut grid_per_patch = 0.0;
        let mut grid_size = 0.0;
        let block_count = reader.read_u32()?;
        for _ in 0..block_count {
            let block_type = reader.read_u32()?;
            let block_offset = reader.read_u32()?;
            let next_block_header_offset = reader.position();
            reader.set_position(block_offset as u64);

            match FromPrimitive::from_u32(block_type) {
                Some(BlockType::ZoneInfo) => {
                    let _zone_type = reader.read_u32()?;
                    let _width = reader.read_u32()?;
                    let _height = reader.read_u32()?;
                    grid_per_patch = reader.read_u32()? as f32;
                    grid_size = reader.read_f32()?;
                }
                Some(BlockType::EventPositions) => {
                    let object_count = reader.read_u32()?;
                    for _ in 0..object_count {
                        let position = reader.read_vector3_f32()?;
                        let name = reader.read_u8_length_string()?;
                        event_positions.push((String::from(name), position));
                    }
                }
                _ => {}
            }

            reader.set_position(next_block_header_offset);
        }

        Ok(Self {
            grid_per_patch,
            grid_size,
            event_positions,
        })
    }
}
