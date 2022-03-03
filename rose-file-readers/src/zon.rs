use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use thiserror::Error;

use crate::{reader::FileReader, types::Vec3, RoseFile};

#[derive(Default)]
pub struct ZonFile {
    pub grid_per_patch: f32,
    pub grid_size: f32,
    pub event_positions: Vec<(String, Vec3<f32>)>,
    pub tile_textures: Vec<String>,
    pub tiles: Vec<ZonTile>,
}

pub struct ZonTile {
    pub layer1: u32,
    pub layer2: u32,
    pub offset1: u32,
    pub offset2: u32,
    pub blend: bool,
    pub rotation: ZonTileRotation,
}

#[derive(Debug, FromPrimitive, PartialEq)]
pub enum ZonTileRotation {
    Unknown = 0,
    None = 1,
    FlipHorizontal = 2,
    FlipVertical = 3,
    Flip = 4,
    Clockwise90 = 5,
    CounterClockwise90 = 6,
}

#[derive(Error, Debug)]
pub enum ZonReadError {
    #[error("Invalid tile rotation")]
    InvalidTileRotation,
}

#[derive(FromPrimitive)]
enum BlockType {
    ZoneInfo = 0,
    EventPositions = 1,
    Textures = 2,
    Tiles = 3,
    Economy = 4,
}

#[derive(Default, Clone, Copy)]
pub struct ZonReadOptions {
    pub skip_zone_info: bool,
    pub skip_event_positions: bool,
    pub skip_textures: bool,
    pub skip_tiles: bool,
}

#[allow(dead_code)]
impl RoseFile for ZonFile {
    type ReadOptions = ZonReadOptions;

    fn read(mut reader: FileReader, read_options: &ZonReadOptions) -> Result<Self, anyhow::Error> {
        let mut event_positions = Vec::new();
        let mut grid_per_patch = 0.0;
        let mut grid_size = 0.0;
        let mut tile_textures = Vec::new();
        let mut tiles = Vec::new();

        let block_count = reader.read_u32()?;
        for _ in 0..block_count {
            let block_type = reader.read_u32()?;
            let block_offset = reader.read_u32()?;
            let next_block_header_offset = reader.position();
            reader.set_position(block_offset as u64);

            match FromPrimitive::from_u32(block_type) {
                Some(BlockType::ZoneInfo) => {
                    if !read_options.skip_zone_info {
                        reader.skip(12);
                        grid_per_patch = reader.read_u32()? as f32;
                        grid_size = reader.read_f32()?;
                        reader.skip(8);
                    }
                }
                Some(BlockType::EventPositions) => {
                    if !read_options.skip_event_positions {
                        let object_count = reader.read_u32()? as usize;
                        event_positions.reserve_exact(object_count);
                        for _ in 0..object_count {
                            let position = reader.read_vector3_f32()?;
                            let name = reader.read_u8_length_string()?;
                            event_positions.push((name.into(), position));
                        }
                    }
                }
                Some(BlockType::Textures) => {
                    if !read_options.skip_textures {
                        let texture_count = reader.read_u32()? as usize;
                        tile_textures.reserve_exact(texture_count);
                        for _ in 0..texture_count {
                            tile_textures.push(reader.read_u8_length_string()?.into());
                        }
                    }
                }
                Some(BlockType::Tiles) => {
                    if !read_options.skip_tiles {
                        let tile_count = reader.read_u32()? as usize;
                        tiles.reserve_exact(tile_count);
                        for _ in 0..tile_count {
                            let layer1 = reader.read_u32()?;
                            let layer2 = reader.read_u32()?;
                            let offset1 = reader.read_u32()?;
                            let offset2 = reader.read_u32()?;
                            let blend = reader.read_u32()? != 0;
                            let rotation = FromPrimitive::from_u32(reader.read_u32()?)
                                .ok_or(ZonReadError::InvalidTileRotation)?;
                            reader.skip(4);
                            tiles.push(ZonTile {
                                layer1,
                                layer2,
                                offset1,
                                offset2,
                                blend,
                                rotation,
                            })
                        }
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
            tile_textures,
            tiles,
        })
    }
}
