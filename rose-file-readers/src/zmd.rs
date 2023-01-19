use crate::{
    types::{Quat4, Vec3},
    RoseFile, RoseFileReader,
};

pub struct ZmdFile {
    pub bones: Vec<ZmdBone>,
    pub dummy_bones: Vec<ZmdBone>,
}

pub struct ZmdBone {
    pub parent: u16,
    pub position: Vec3<f32>,
    pub rotation: Quat4<f32>,
}

impl RoseFile for ZmdFile {
    type ReadOptions = ();
    type WriteOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let magic = reader.read_fixed_length_string(7)?;
        let version = if magic == "ZMD0002" {
            2
        } else if magic == "ZMD0003" {
            3
        } else {
            return Err(anyhow::anyhow!("Invalid ZMD magic header: {}", magic));
        };

        let bone_count = reader.read_u32()? as usize;
        let mut bones = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let parent = reader.read_u32()? as u16;
            let _name = reader.read_null_terminated_string()?;
            let position = reader.read_vector3_f32()?;
            let rotation = reader.read_quat4_wxyz_f32()?;
            bones.push(ZmdBone {
                parent,
                position,
                rotation,
            });
        }

        let dummy_bone_count = reader.read_u32()? as usize;
        let mut dummy_bones = Vec::with_capacity(dummy_bone_count);
        for _ in 0..dummy_bone_count {
            let _name = reader.read_null_terminated_string()?;
            let parent = reader.read_u32()? as u16;
            let position = reader.read_vector3_f32()?;
            let rotation = if version == 2 {
                Quat4 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 1.0,
                }
            } else {
                reader.read_quat4_wxyz_f32()?
            };

            dummy_bones.push(ZmdBone {
                parent,
                position,
                rotation,
            });
        }

        Ok(Self { bones, dummy_bones })
    }
}
