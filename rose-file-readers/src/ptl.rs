use std::ops::RangeInclusive;

use anyhow::bail;

use crate::{reader::RoseFileReader, RoseFile, VfsPathBuf};

#[derive(Debug)]
pub enum PtlKeyframeData {
    SizeXY(RangeInclusive<f32>, RangeInclusive<f32>),
    Timer(RangeInclusive<f32>),
    Red(RangeInclusive<f32>),
    Green(RangeInclusive<f32>),
    Blue(RangeInclusive<f32>),
    Alpha(RangeInclusive<f32>),
    ColourRGBA(
        RangeInclusive<f32>,
        RangeInclusive<f32>,
        RangeInclusive<f32>,
        RangeInclusive<f32>,
    ),
    VelocityX(RangeInclusive<f32>),
    VelocityY(RangeInclusive<f32>),
    VelocityZ(RangeInclusive<f32>),
    VelocityXYZ(
        RangeInclusive<f32>,
        RangeInclusive<f32>,
        RangeInclusive<f32>,
    ),
    Texture(RangeInclusive<f32>),
    Rotation(RangeInclusive<f32>),
}

#[derive(Debug)]
pub struct PtlKeyframe {
    pub start_time: RangeInclusive<f32>,
    pub fade: bool,
    pub data: PtlKeyframeData,
}

#[derive(Copy, Clone, Debug)]
pub enum PtlUpdateCoords {
    World,
    LocalPosition,
    Local,
}

#[derive(Debug)]
pub struct PtlSequence {
    pub name: String,
    pub life: RangeInclusive<f32>,
    pub emit_rate: RangeInclusive<f32>,
    pub num_loops: i32,
    pub emit_radius_x: RangeInclusive<f32>,
    pub emit_radius_y: RangeInclusive<f32>,
    pub emit_radius_z: RangeInclusive<f32>,
    pub gravity_x: RangeInclusive<f32>,
    pub gravity_y: RangeInclusive<f32>,
    pub gravity_z: RangeInclusive<f32>,
    pub texture_path: VfsPathBuf,
    pub num_particles: i32,
    pub align_type: u32,
    pub update_coords: PtlUpdateCoords,
    pub texture_atlas_cols: u32,
    pub texture_atlas_rows: u32,
    pub dst_blend_mode: u32,
    pub src_blend_mode: u32,
    pub blend_op: u32,
    pub keyframes: Vec<PtlKeyframe>,
}

#[derive(Debug)]
pub struct PtlFile {
    pub sequences: Vec<PtlSequence>,
}

impl RoseFile for PtlFile {
    type ReadOptions = ();
    type WriteOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let num_sequences = reader.read_u32()? as usize;
        let mut sequences = Vec::with_capacity(num_sequences);

        for _ in 0..num_sequences {
            let name = reader.read_u32_length_string()?.to_string();
            let life_min = reader.read_f32()?;
            let life_max = reader.read_f32()?;
            let emit_rate_min = reader.read_f32()?;
            let emit_rate_max = reader.read_f32()?;
            let num_loops = reader.read_i32()?;
            let _spawn_direction_min = reader.read_vector3_f32()?; // Unused
            let _spawn_direction_max = reader.read_vector3_f32()?; // Unused
            let emit_radius_min = reader.read_vector3_f32()?;
            let emit_radius_max = reader.read_vector3_f32()?;
            let gravity_min = reader.read_vector3_f32()?;
            let gravity_max = reader.read_vector3_f32()?;
            let texture_path = VfsPathBuf::new(&reader.read_u32_length_string()?);
            let num_particles = reader.read_i32()?;
            let align_type = reader.read_u32()?;
            let update_coords = match reader.read_u32()? {
                0 => PtlUpdateCoords::World,
                1 => PtlUpdateCoords::LocalPosition,
                2 => PtlUpdateCoords::Local,
                _ => PtlUpdateCoords::Local,
            };
            let texture_atlas_cols = reader.read_u32()?;
            let texture_atlas_rows = reader.read_u32()?;
            let _sprite_type = reader.read_u32()?; // Unused
            let dst_blend_mode = reader.read_u32()?;
            let src_blend_mode = reader.read_u32()?;
            let blend_op = reader.read_u32()?;

            let num_keyframes = reader.read_u32()? as usize;
            let mut keyframes = Vec::with_capacity(num_keyframes);
            for _ in 0..num_keyframes {
                let keyframe_type = reader.read_u32()?;
                let start_time_min = reader.read_f32()?;
                let start_time_max = reader.read_f32()?;
                let fade = reader.read_u8()? != 0;

                let data = match keyframe_type {
                    1 => {
                        let size_x_min = reader.read_f32()?;
                        let size_y_min = reader.read_f32()?;
                        let size_x_max = reader.read_f32()?;
                        let size_y_max = reader.read_f32()?;
                        PtlKeyframeData::SizeXY(size_x_min..=size_x_max, size_y_min..=size_y_max)
                    }
                    2 => {
                        let timer_min = reader.read_f32()?;
                        let timer_max = reader.read_f32()?;
                        PtlKeyframeData::Timer(timer_min..=timer_max)
                    }
                    3 => {
                        let red_min = reader.read_f32()?;
                        let red_max = reader.read_f32()?;
                        PtlKeyframeData::Red(red_min..=red_max)
                    }
                    4 => {
                        let green_min = reader.read_f32()?;
                        let green_max = reader.read_f32()?;
                        PtlKeyframeData::Green(green_min..=green_max)
                    }
                    5 => {
                        let blue_min = reader.read_f32()?;
                        let blue_max = reader.read_f32()?;
                        PtlKeyframeData::Blue(blue_min..=blue_max)
                    }
                    6 => {
                        let alpha_min = reader.read_f32()?;
                        let alpha_max = reader.read_f32()?;
                        PtlKeyframeData::Alpha(alpha_min..=alpha_max)
                    }
                    7 => {
                        let red_min = reader.read_f32()?;
                        let green_min = reader.read_f32()?;
                        let blue_min = reader.read_f32()?;
                        let alpha_min = reader.read_f32()?;

                        let red_max = reader.read_f32()?;
                        let green_max = reader.read_f32()?;
                        let blue_max = reader.read_f32()?;
                        let alpha_max = reader.read_f32()?;
                        PtlKeyframeData::ColourRGBA(
                            red_min..=red_max,
                            green_min..=green_max,
                            blue_min..=blue_max,
                            alpha_min..=alpha_max,
                        )
                    }
                    8 => {
                        let velocity_x_min = reader.read_f32()?;
                        let velocity_x_max = reader.read_f32()?;
                        PtlKeyframeData::VelocityX(velocity_x_min..=velocity_x_max)
                    }
                    9 => {
                        let velocity_y_min = reader.read_f32()?;
                        let velocity_y_max = reader.read_f32()?;
                        PtlKeyframeData::VelocityY(velocity_y_min..=velocity_y_max)
                    }
                    10 => {
                        let velocity_z_min = reader.read_f32()?;
                        let velocity_z_max = reader.read_f32()?;
                        PtlKeyframeData::VelocityZ(velocity_z_min..=velocity_z_max)
                    }
                    11 => {
                        let velocity_x_min = reader.read_f32()?;
                        let velocity_y_min = reader.read_f32()?;
                        let velocity_z_min = reader.read_f32()?;

                        let velocity_x_max = reader.read_f32()?;
                        let velocity_y_max = reader.read_f32()?;
                        let velocity_z_max = reader.read_f32()?;
                        PtlKeyframeData::VelocityXYZ(
                            velocity_x_min..=velocity_x_max,
                            velocity_y_min..=velocity_y_max,
                            velocity_z_min..=velocity_z_max,
                        )
                    }
                    12 => {
                        let texture_min = reader.read_f32()?;
                        let texture_max = reader.read_f32()?;
                        PtlKeyframeData::Texture(texture_min..=texture_max)
                    }
                    13 => {
                        let rotation_min = reader.read_f32()?;
                        let rotation_max = reader.read_f32()?;
                        PtlKeyframeData::Rotation(rotation_min..=rotation_max)
                    }
                    invalid => bail!("Invalid keyframe type {}", invalid),
                };

                keyframes.push(PtlKeyframe {
                    start_time: start_time_min..=start_time_max.max(start_time_min),
                    fade,
                    data,
                })
            }

            sequences.push(PtlSequence {
                name,
                life: life_min..=life_max.max(life_min),
                emit_rate: emit_rate_min..=emit_rate_max.max(emit_rate_min),
                num_loops,
                emit_radius_x: emit_radius_min.x..=emit_radius_max.x.max(emit_radius_min.x),
                emit_radius_y: emit_radius_min.y..=emit_radius_max.y.max(emit_radius_min.y),
                emit_radius_z: emit_radius_min.z..=emit_radius_max.z.max(emit_radius_min.z),
                gravity_x: gravity_min.x..=gravity_max.x.max(gravity_min.x),
                gravity_y: gravity_min.y..=gravity_max.y.max(gravity_min.y),
                gravity_z: gravity_min.z..=gravity_max.z.max(gravity_min.z),
                texture_path,
                num_particles,
                align_type,
                update_coords,
                texture_atlas_cols,
                texture_atlas_rows,
                dst_blend_mode,
                src_blend_mode,
                blend_op,
                keyframes,
            })
        }

        Ok(Self { sequences })
    }
}
