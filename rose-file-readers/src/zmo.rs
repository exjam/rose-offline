use std::time::Duration;

use anyhow::bail;

use crate::{
    reader::RoseFileReader,
    types::{Quat4, Vec2, Vec3},
    RoseFile,
};

pub struct ZmoFile {
    pub fps: usize,
    pub num_frames: usize,
    pub channels: Vec<(u32, ZmoChannel)>,
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

pub enum ZmoChannel {
    Empty,
    Position(Vec<Vec3<f32>>),
    Rotation(Vec<Quat4<f32>>),
    Normal(Vec<Vec3<f32>>),
    Alpha(Vec<f32>),
    UV1(Vec<Vec2<f32>>),
    UV2(Vec<Vec2<f32>>),
    UV3(Vec<Vec2<f32>>),
    UV4(Vec<Vec2<f32>>),
    Texture(Vec<f32>),
    Scale(Vec<f32>),
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
        let mut channels = Vec::new();

        if !read_options.skip_animation {
            let channel_count = reader.read_u32()? as usize;
            channels.reserve_exact(channel_count);
            for _ in 0..channel_count {
                let channel_type = reader.read_u32()?;
                let channel_bone_index = reader.read_u32()?;
                let channel = match channel_type {
                    1 => ZmoChannel::Empty,
                    2 => ZmoChannel::Position(Vec::with_capacity(num_frames)),
                    4 => ZmoChannel::Rotation(Vec::with_capacity(num_frames)),
                    8 => ZmoChannel::Normal(Vec::with_capacity(num_frames)),
                    16 => ZmoChannel::Alpha(Vec::with_capacity(num_frames)),
                    32 => ZmoChannel::UV1(Vec::with_capacity(num_frames)),
                    64 => ZmoChannel::UV2(Vec::with_capacity(num_frames)),
                    128 => ZmoChannel::UV3(Vec::with_capacity(num_frames)),
                    256 => ZmoChannel::UV4(Vec::with_capacity(num_frames)),
                    512 => ZmoChannel::Texture(Vec::with_capacity(num_frames)),
                    1024 => ZmoChannel::Scale(Vec::with_capacity(num_frames)),
                    invalid => bail!("Invalid ZMO channel type: {}", invalid),
                };
                channels.push((channel_bone_index, channel));
            }

            for _ in 0..num_frames {
                for channel in channels.iter_mut() {
                    match &mut channel.1 {
                        ZmoChannel::Empty => {}
                        ZmoChannel::Position(values) | ZmoChannel::Normal(values) => {
                            values.push(reader.read_vector3_f32()?);
                        }
                        ZmoChannel::Rotation(values) => {
                            values.push(reader.read_quat4_wxyz_f32()?);
                        }
                        ZmoChannel::UV1(values)
                        | ZmoChannel::UV2(values)
                        | ZmoChannel::UV3(values)
                        | ZmoChannel::UV4(values) => {
                            values.push(reader.read_vector2_f32()?);
                        }
                        ZmoChannel::Alpha(values)
                        | ZmoChannel::Texture(values)
                        | ZmoChannel::Scale(values) => {
                            values.push(reader.read_f32()?);
                        }
                    }
                }
            }
        }

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
            channels,
            frame_events,
            total_attack_frames,
        })
    }
}
