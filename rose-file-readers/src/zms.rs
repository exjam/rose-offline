use crate::{reader::RoseFileReader, RoseFile};
use anyhow::anyhow;
use thiserror::Error;

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ZmsFormatFlags: u32 {
        const POSITION = (1 << 1);
        const NORMAL = (1 << 2);
        const COLOR = (1 << 3);
        const BONE_WEIGHT = (1 << 4);
        const BONE_INDEX = (1 << 5);
        const TANGENT = (1 << 6);
        const UV1 = (1 << 7);
        const UV2 = (1 << 8);
        const UV3 = (1 << 9);
        const UV4 = (1 << 10);
    }
}

pub struct ZmsFile {
    pub format: ZmsFormatFlags,
    pub position: Vec<[f32; 3]>,
    pub normal: Vec<[f32; 3]>,
    pub color: Vec<[f32; 4]>,
    pub bone_weights: Vec<[f32; 4]>,
    pub bone_indices: Vec<[u32; 4]>,
    pub tangent: Vec<[f32; 3]>,
    pub uv1: Vec<[f32; 2]>,
    pub uv2: Vec<[f32; 2]>,
    pub uv3: Vec<[f32; 2]>,
    pub uv4: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
}

#[derive(Error, Debug)]
pub enum ZmsReadError {
    #[error("Invalid bone index")]
    InvalidBoneIndex,
}

impl RoseFile for ZmsFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let magic = reader.read_null_terminated_string()?;
        if magic == "ZMS0006" {
            Self::read_version6(&mut reader)
        } else if magic == "ZMS0007" || magic == "ZMS0008" {
            Self::read_version8(&mut reader)
        } else {
            Err(anyhow!("Invalid ZMS magic header: {}", magic))
        }
    }
}

impl ZmsFile {
    fn read_version6(reader: &mut RoseFileReader) -> Result<Self, anyhow::Error> {
        let format_bits = reader.read_u32()?;
        let format = ZmsFormatFlags::from_bits(format_bits)
            .ok_or_else(|| anyhow!("Invalid ZMS format bits: {:X}", format_bits))?;
        let _bb_min = reader.read_vector3_f32()?;
        let _bb_max = reader.read_vector3_f32()?;

        let bone_count = reader.read_u32()?;
        let mut bones = Vec::new();
        for _ in 0..bone_count {
            let _ = reader.read_u32()?;
            bones.push(reader.read_u32()? as u16);
        }

        let vertex_count = reader.read_u32()? as usize;

        let read_vertex_f32x2 =
            |vertex_count, reader: &mut RoseFileReader| -> Result<Vec<[f32; 2]>, anyhow::Error> {
                let mut values = Vec::with_capacity(vertex_count);
                for _ in 0..vertex_count {
                    let _vertex_id = reader.read_u32()?;
                    let value_x = reader.read_f32()?;
                    let value_y = reader.read_f32()?;
                    values.push([value_x, value_y]);
                }
                Ok(values)
            };
        let read_vertex_f32x3 =
            |vertex_count, reader: &mut RoseFileReader| -> Result<Vec<[f32; 3]>, anyhow::Error> {
                let mut values = Vec::with_capacity(vertex_count);
                for _ in 0..vertex_count {
                    let _vertex_id = reader.read_u32()?;
                    let value = reader.read_vector3_f32()?;
                    values.push([value.x, value.y, value.z]);
                }
                Ok(values)
            };
        let read_vertex_f32x4 =
            |vertex_count, reader: &mut RoseFileReader| -> Result<Vec<[f32; 4]>, anyhow::Error> {
                let mut values = Vec::with_capacity(vertex_count);
                for _ in 0..vertex_count {
                    let _vertex_id = reader.read_u32()?;
                    let value = reader.read_vector4_f32()?;
                    values.push([value.x, value.y, value.z, value.w]);
                }
                Ok(values)
            };

        let position = if format.contains(ZmsFormatFlags::POSITION) {
            read_vertex_f32x3(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let normal = if format.contains(ZmsFormatFlags::NORMAL) {
            read_vertex_f32x3(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let color = if format.contains(ZmsFormatFlags::COLOR) {
            read_vertex_f32x4(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let (bone_weights, bone_indices) = if format.contains(ZmsFormatFlags::BONE_WEIGHT)
            && format.contains(ZmsFormatFlags::BONE_INDEX)
        {
            let mut bone_weights = Vec::with_capacity(vertex_count);
            let mut bone_indices = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                let _vertex_id = reader.read_u32()?;
                let weight = reader.read_vector4_f32()?;
                let index = reader.read_vector4_u32()?;
                bone_weights.push([weight.x, weight.y, weight.z, weight.w]);

                let bone_x = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_y = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_z = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_w = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                bone_indices.push([bone_x as u32, bone_y as u32, bone_z as u32, bone_w as u32]);
            }
            (bone_weights, bone_indices)
        } else {
            (Vec::new(), Vec::new())
        };

        let tangent = if format.contains(ZmsFormatFlags::TANGENT) {
            read_vertex_f32x3(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let uv1 = if format.contains(ZmsFormatFlags::UV1) {
            read_vertex_f32x2(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let uv2 = if format.contains(ZmsFormatFlags::UV2) {
            read_vertex_f32x2(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let uv3 = if format.contains(ZmsFormatFlags::UV3) {
            read_vertex_f32x2(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let uv4 = if format.contains(ZmsFormatFlags::UV4) {
            read_vertex_f32x2(vertex_count, reader)?
        } else {
            Vec::new()
        };

        let triangle_count = reader.read_u32()? as usize;
        let mut indices = Vec::with_capacity(triangle_count);
        for _ in 0..triangle_count {
            let _vertex_id = reader.read_u32()?;
            indices.push(reader.read_u32()? as u16);
            indices.push(reader.read_u32()? as u16);
            indices.push(reader.read_u32()? as u16);
        }

        Ok(Self {
            format,
            position,
            normal,
            color,
            bone_weights,
            bone_indices,
            tangent,
            uv1,
            uv2,
            uv3,
            uv4,
            indices,
        })
    }

    fn read_version8(reader: &mut RoseFileReader) -> Result<Self, anyhow::Error> {
        let format_bits = reader.read_u32()?;
        let format = ZmsFormatFlags::from_bits(format_bits)
            .ok_or_else(|| anyhow!("Invalid ZMS format bits: {:X}", format_bits))?;
        let _bb_min = reader.read_vector3_f32()?;
        let _bb_max = reader.read_vector3_f32()?;

        let bone_count = reader.read_u16()?;
        let mut bones = Vec::new();
        for _ in 0..bone_count {
            bones.push(reader.read_u16()?);
        }

        let vertex_count = reader.read_u16()? as usize;

        let position = if format.contains(ZmsFormatFlags::POSITION) {
            reader.read_vec::<[f32; 3]>(vertex_count)?
        } else {
            Vec::new()
        };

        let normal = if format.contains(ZmsFormatFlags::NORMAL) {
            reader.read_vec::<[f32; 3]>(vertex_count)?
        } else {
            Vec::new()
        };

        let color = if format.contains(ZmsFormatFlags::COLOR) {
            reader.read_vec::<[f32; 4]>(vertex_count)?
        } else {
            Vec::new()
        };

        let (bone_weights, bone_indices) = if format.contains(ZmsFormatFlags::BONE_WEIGHT)
            && format.contains(ZmsFormatFlags::BONE_INDEX)
        {
            let mut bone_weights = Vec::with_capacity(vertex_count);
            let mut bone_indices = Vec::with_capacity(vertex_count);
            for _ in 0..vertex_count {
                let weight = reader.read_vector4_f32()?;
                let index = reader.read_vector4_u16()?;
                bone_weights.push([weight.x, weight.y, weight.z, weight.w]);

                let bone_x = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_y = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_z = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                let bone_w = bones
                    .get(index.x as usize)
                    .cloned()
                    .ok_or(ZmsReadError::InvalidBoneIndex)?;
                bone_indices.push([bone_x as u32, bone_y as u32, bone_z as u32, bone_w as u32]);
            }
            (bone_weights, bone_indices)
        } else {
            (Vec::new(), Vec::new())
        };

        let tangent = if format.contains(ZmsFormatFlags::TANGENT) {
            reader.read_vec::<[f32; 3]>(vertex_count)?
        } else {
            Vec::new()
        };

        let uv1 = if format.contains(ZmsFormatFlags::UV1) {
            reader.read_vec::<[f32; 2]>(vertex_count)?
        } else {
            Vec::new()
        };

        let uv2 = if format.contains(ZmsFormatFlags::UV2) {
            reader.read_vec::<[f32; 2]>(vertex_count)?
        } else {
            Vec::new()
        };

        let uv3 = if format.contains(ZmsFormatFlags::UV3) {
            reader.read_vec::<[f32; 2]>(vertex_count)?
        } else {
            Vec::new()
        };

        let uv4 = if format.contains(ZmsFormatFlags::UV4) {
            reader.read_vec::<[f32; 2]>(vertex_count)?
        } else {
            Vec::new()
        };

        let triangle_count = reader.read_u16()? as usize;
        let indices = reader.read_vec::<u16>(triangle_count * 3)?;

        Ok(Self {
            format,
            position,
            normal,
            color,
            bone_weights,
            bone_indices,
            tangent,
            uv1,
            uv2,
            uv3,
            uv4,
            indices,
        })
    }
}
