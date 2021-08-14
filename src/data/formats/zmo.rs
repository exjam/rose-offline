use std::time::Duration;

use crate::data::formats::reader::{FileReader, ReadError};

pub struct ZmoFile {
    pub fps: usize,
    pub num_frames: usize,
    pub frame_events: Vec<u16>,
    pub total_attack_frames: usize,
}

#[derive(Debug)]
pub enum ZmoReadError {
    InvalidMagic,
    UnexpectedEof,
}

impl From<ReadError> for ZmoReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => ZmoReadError::UnexpectedEof,
        }
    }
}

#[allow(dead_code)]
impl ZmoFile {
    pub fn read(mut reader: FileReader) -> Result<Self, ZmoReadError> {
        let magic = reader.read_null_terminated_string()?;
        if magic != "ZMO0002" {
            return Err(ZmoReadError::InvalidMagic);
        }

        let fps = reader.read_u32()? as usize;
        let num_frames = reader.read_u32()? as usize;

        // TODO: There is the animation data here, which we do not need for server

        let mut frame_events = Vec::new();
        let mut total_attack_frames = 0;
        reader.set_position_from_end(-4);
        if let Ok(extended_magic) = reader.read_fixed_length_string(4) {
            if extended_magic == "EZMO" || extended_magic == "3ZMO" {
                reader.set_position_from_end(-8);
                let position = reader.read_u32()? as u64;
                reader.set_position(position);

                let num_frame_events = reader.read_u16()?;
                for _ in 0..num_frame_events {
                    let frame_event = reader.read_u16()?;
                    frame_events.push(frame_event);

                    match frame_event {
                        10 | 20..=28 | 56..=57 | 66..=67 => total_attack_frames += 1,
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            fps,
            num_frames,
            frame_events,
            total_attack_frames,
        })
    }

    pub fn get_duration(&self) -> Duration {
        Duration::from_nanos((self.num_frames as u64 * 1_000_000_000) / self.fps as u64)
    }
}
