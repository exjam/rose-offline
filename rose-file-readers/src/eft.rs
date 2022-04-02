use crate::{reader::RoseFileReader, types::Vec3, RoseFile, VfsPathBuf};

#[derive(Debug)]
pub struct EftParticle {
    pub particle_file: VfsPathBuf,

    pub animation_file: Option<VfsPathBuf>,
    pub animation_repeat_count: u32,
    pub position: Vec3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub start_delay: u32,
    pub is_linked: bool,
}

#[derive(Debug)]
pub struct EftMesh {
    pub mesh_file: VfsPathBuf,
    pub mesh_animation_file: Option<VfsPathBuf>,
    pub mesh_texture_file: VfsPathBuf,

    pub alpha_enabled: bool,
    pub two_sided: bool,
    pub alpha_test_enabled: bool,
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub src_blend_factor: u32,
    pub dst_blend_factor: u32,
    pub blend_op: u32,

    pub animation_file: Option<VfsPathBuf>,
    pub animation_repeat_count: u32,
    pub position: Vec3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub start_delay: u32,
    pub repeat_count: u32,
    pub is_linked: bool,
}

#[derive(Debug)]
pub struct EftFile {
    pub sound_file: Option<VfsPathBuf>,
    pub sound_repeat_count: u32,
    pub particles: Vec<EftParticle>,
    pub meshes: Vec<EftMesh>,
}

impl RoseFile for EftFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let skip_len = reader.read_u32()? as u64;
        reader.skip(skip_len);
        let use_sound_file = reader.read_u32()? != 0;
        let sound_file = VfsPathBuf::new(&reader.read_u32_length_string()?);
        let sound_repeat_count = reader.read_u32()?;

        let num_particles = reader.read_u32()? as usize;
        let mut particles = Vec::with_capacity(num_particles);
        for _ in 0..num_particles {
            let skip_len = reader.read_u32()? as u64;
            reader.skip(skip_len);
            let skip_len = reader.read_u32()? as u64;
            reader.skip(skip_len);
            reader.skip(4);

            let particle_file = VfsPathBuf::new(&reader.read_u32_length_string()?);

            let use_animation_file = reader.read_u32()? != 0;
            let animation_file = VfsPathBuf::new(&reader.read_u32_length_string()?);
            let animation_repeat_count = reader.read_u32()?;
            reader.skip(4);

            let position = reader.read_vector3_f32()?;
            let pitch = reader.read_f32()?;
            let yaw = reader.read_f32()?;
            let roll = reader.read_f32()?;
            reader.skip(4);

            let start_delay = reader.read_u32()?;
            let is_linked = reader.read_u32()? != 0;

            particles.push(EftParticle {
                particle_file,
                animation_file: if use_animation_file
                    && !animation_file.path().as_os_str().is_empty()
                    && animation_file.path().as_os_str() != "NULL"
                {
                    Some(animation_file)
                } else {
                    None
                },
                animation_repeat_count,
                position,
                pitch,
                yaw,
                roll,
                start_delay,
                is_linked,
            });
        }

        let num_meshes = reader.read_u32()? as usize;
        let mut meshes = Vec::with_capacity(num_meshes);
        for _ in 0..num_meshes {
            let skip_len = reader.read_u32()? as u64;
            reader.skip(skip_len);
            let skip_len = reader.read_u32()? as u64;
            reader.skip(skip_len);
            reader.skip(4);

            let mesh_file = VfsPathBuf::new(&reader.read_u32_length_string()?);
            let mesh_animation_file = VfsPathBuf::new(&reader.read_u32_length_string()?);
            let mesh_texture_file = VfsPathBuf::new(&reader.read_u32_length_string()?);

            let alpha_enabled = reader.read_u32()? != 0;
            let two_sided = reader.read_u32()? != 0;
            let alpha_test_enabled = reader.read_u32()? != 0;
            let depth_test_enabled = reader.read_u32()? != 0;
            let depth_write_enabled = reader.read_u32()? != 0;
            let src_blend_factor = reader.read_u32()?;
            let dst_blend_factor = reader.read_u32()?;
            let blend_op = reader.read_u32()?;

            let use_animation_file = reader.read_u32()? != 0;
            let animation_file = VfsPathBuf::new(&reader.read_u32_length_string()?);
            let animation_repeat_count = reader.read_u32()?;
            reader.skip(4);

            let position = reader.read_vector3_f32()?;
            let pitch = reader.read_f32()?;
            let yaw = reader.read_f32()?;
            let roll = reader.read_f32()?;
            reader.skip(4);

            let start_delay = reader.read_u32()?;
            let repeat_count = reader.read_u32()?;
            let is_linked = reader.read_u32()? != 0;

            meshes.push(EftMesh {
                mesh_file,
                mesh_animation_file: if !mesh_animation_file.path().as_os_str().is_empty()
                    && mesh_animation_file.path().as_os_str() != "NULL"
                {
                    Some(mesh_animation_file)
                } else {
                    None
                },
                mesh_texture_file,
                alpha_enabled,
                two_sided,
                alpha_test_enabled,
                depth_test_enabled,
                depth_write_enabled,
                src_blend_factor,
                dst_blend_factor,
                blend_op,
                animation_file: if use_animation_file
                    && !animation_file.path().as_os_str().is_empty()
                    && animation_file.path().as_os_str() != "NULL"
                {
                    Some(animation_file)
                } else {
                    None
                },
                animation_repeat_count,
                position,
                pitch,
                yaw,
                roll,
                start_delay,
                repeat_count,
                is_linked,
            });
        }

        Ok(Self {
            sound_file: if use_sound_file
                && !sound_file.path().as_os_str().is_empty()
                && sound_file.path().as_os_str() != "NULL"
            {
                Some(sound_file)
            } else {
                None
            },
            sound_repeat_count,
            particles,
            meshes,
        })
    }
}
