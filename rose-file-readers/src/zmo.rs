use std::time::Duration;

use crate::{reader::RoseFileReader, RoseFile};

pub struct ZmoFile {
    pub fps: usize,
    pub num_frames: usize,
    pub frame_events: Vec<u16>,
    pub total_attack_frames: usize,
}

#[derive(Default)]
pub struct ZmoReadOptions {
    pub skip_animation: bool,
}

impl ZmoFile {
    pub fn get_duration(&self) -> Duration {
        Duration::from_nanos((self.num_frames as u64 * 1_000_000_000) / self.fps as u64)
    }
}

impl RoseFile for ZmoFile {
    type ReadOptions = ZmoReadOptions;

    fn read(
        mut reader: RoseFileReader,
        read_options: &ZmoReadOptions,
    ) -> Result<Self, anyhow::Error> {
        let magic = reader.read_null_terminated_string()?;
        if magic != "ZMO0002" {
            return Err(anyhow::anyhow!("Invalid ZMO magic header: {}", magic));
        }

        let fps = reader.read_u32()? as usize;
        let num_frames = reader.read_u32()? as usize;

        // We do not need the actual animation data when reading for server
        if read_options.skip_animation {}

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
}
