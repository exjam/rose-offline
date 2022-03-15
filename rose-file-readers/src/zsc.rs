use std::num::NonZeroU16;

use anyhow::{anyhow, bail};

use crate::{
    reader::RoseFileReader,
    types::{Vec3, Vec4},
    vfs::VfsPathBuf,
    RoseFile,
};

#[derive(Copy, Clone, Debug)]
pub enum ZscMaterialBlend {
    Normal,
    Lighten,
}

#[derive(Copy, Clone, Debug)]
pub enum ZscMaterialGlow {
    Simple(Vec3<f32>),
    Light(Vec3<f32>),
    Texture(Vec3<f32>),
    TextureLight(Vec3<f32>),
    Alpha(Vec3<f32>),
}

#[derive(Clone, Debug)]
pub struct ZscMaterial {
    pub path: VfsPathBuf,
    pub is_skin: bool,
    pub alpha_enabled: bool,
    pub two_sided: bool,
    pub alpha_test: Option<f32>,
    pub z_write_enabled: bool,
    pub z_test_enabled: bool,
    pub blend_mode: ZscMaterialBlend,
    pub specular_enabled: bool,
    pub alpha: f32,
    pub glow: Option<ZscMaterialGlow>,
}

#[derive(Copy, Clone, Debug)]
pub enum ZscCollisionShape {
    Sphere,
    AxisAlignedBoundingBox,
    ObjectOrientedBoundingBox,
    Polygon,
}

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ZscCollisionFlags: u32 {
        const NOT_MOVEABLE = (1 << 3);
        const NOT_PICKABLE = (1 << 4);
        const HEIGHT_ONLY = (1 << 5);
        const NOT_CAMERA_COLLISION = (1 << 6);
    }
}

#[derive(Clone, Debug)]
pub struct ZscObjectPart {
    pub mesh_id: u16,
    pub material_id: u16,
    pub position: Vec3<f32>,
    pub rotation: Vec4<f32>,
    pub scale: Vec3<f32>,
    pub bone_index: Option<u16>,
    pub dummy_index: Option<u16>,
    pub parent: Option<u16>,
    pub collision_shape: Option<ZscCollisionShape>,
    pub collision_flags: ZscCollisionFlags,
    pub animation_path: Option<VfsPathBuf>,
}

#[derive(Copy, Clone, Debug)]
pub enum ZscEffectType {
    Normal,
    DayNight,
    LightContainer,
    Unknown(u16),
}

#[derive(Clone, Debug)]
pub struct ZscObjectEffect {
    pub effect_id: u16,
    pub effect_type: ZscEffectType,
    pub position: Vec3<f32>,
    pub rotation: Vec4<f32>,
    pub scale: Vec3<f32>,
    pub parent: Option<u16>,
}

#[derive(Clone, Debug)]
pub struct ZscObject {
    pub parts: Vec<ZscObjectPart>,
    pub effects: Vec<ZscObjectEffect>,
}

#[derive(Clone, Debug)]
pub struct ZscFile {
    pub meshes: Vec<VfsPathBuf>,
    pub materials: Vec<ZscMaterial>,
    pub effects: Vec<VfsPathBuf>,
    pub objects: Vec<ZscObject>,
}

impl RoseFile for ZscFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let mesh_count = reader.read_u16()? as usize;
        let mut meshes = Vec::with_capacity(mesh_count);
        for _ in 0..mesh_count {
            meshes.push(VfsPathBuf::new(&reader.read_null_terminated_string()?));
        }

        let material_count = reader.read_u16()? as usize;
        let mut materials = Vec::with_capacity(material_count);
        for _ in 0..material_count {
            let path = reader.read_null_terminated_string()?;
            let is_skin = reader.read_u16()? != 0;
            let alpha_enabled = reader.read_u16()? != 0;
            let two_sided = reader.read_u16()? != 0;
            let alpha_test_enabled = reader.read_u16()? != 0;
            let alpha_ref = (reader.read_u16()? as f32) / 256.0;
            let z_test_enabled = reader.read_u16()? != 0;
            let z_write_enabled = reader.read_u16()? != 0;
            let blend_mode = match reader.read_u16()? {
                0 => ZscMaterialBlend::Normal,
                1 => ZscMaterialBlend::Lighten,
                invalid => bail!("Invalid ZscMaterialBlend {}", invalid),
            };
            let specular_enabled = reader.read_u16()? != 0;
            let alpha = reader.read_f32()?;
            let glow_type = reader.read_u16()?;
            let glow_color = reader.read_vector3_f32()?;

            materials.push(ZscMaterial {
                path: VfsPathBuf::new(&path),
                is_skin,
                alpha_enabled,
                two_sided,
                alpha_test: if alpha_test_enabled {
                    Some(alpha_ref)
                } else {
                    None
                },
                z_write_enabled,
                z_test_enabled,
                blend_mode,
                specular_enabled,
                alpha,
                glow: match glow_type {
                    0 | 1 => None,
                    2 => Some(ZscMaterialGlow::Simple(glow_color)),
                    3 => Some(ZscMaterialGlow::Light(glow_color)),
                    4 => Some(ZscMaterialGlow::TextureLight(glow_color)),
                    5 => Some(ZscMaterialGlow::Alpha(glow_color)),
                    invalid => bail!("Invalid ZscMaterialGlow {}", invalid),
                },
            });
        }

        let effect_count = reader.read_u16()? as usize;
        let mut effects = Vec::with_capacity(effect_count);
        for _ in 0..effect_count {
            effects.push(VfsPathBuf::new(&reader.read_null_terminated_string()?));
        }

        let object_count = reader.read_u16()? as usize;
        let mut objects = Vec::with_capacity(object_count);
        for _ in 0..object_count {
            reader.skip(4 * 3);

            let object_part_count = reader.read_u16()? as usize;
            if object_part_count == 0 {
                objects.push(ZscObject {
                    parts: Vec::new(),
                    effects: Vec::new(),
                });
                continue;
            }

            let mut object_parts = Vec::with_capacity(object_part_count);
            for _ in 0..object_part_count {
                let mesh_id = reader.read_u16()?;
                let material_id = reader.read_u16()?;
                let mut position = None;
                let mut rotation = None;
                let mut scale = None;
                let mut bone_index = None;
                let mut dummy_index = None;
                let mut parent = None;
                let mut collision_shape = None;
                let mut collision_flags = ZscCollisionFlags::from_bits_truncate(0);
                let mut animation_path = None;

                loop {
                    let property_id = reader.read_u8()?;
                    if property_id == 0 {
                        break;
                    }
                    let size = reader.read_u8()?;

                    match property_id {
                        1 => position = Some(reader.read_vector3_f32()?),
                        2 => {
                            let w = reader.read_f32()?;
                            let x = reader.read_f32()?;
                            let y = reader.read_f32()?;
                            let z = reader.read_f32()?;
                            rotation = Some(Vec4::<f32> { x, y, z, w });
                        }
                        3 => scale = Some(reader.read_vector3_f32()?),
                        4 => reader.skip(4 * 4),
                        5 => bone_index = Some(reader.read_u16()?),
                        6 => dummy_index = Some(reader.read_u16()?),
                        7 => parent = NonZeroU16::new(reader.read_u16()?).map(|id| id.get() - 1),
                        8..=28 => todo!(),
                        29 => {
                            let bits = reader.read_u16()?;
                            collision_shape = match bits & 0b111 {
                                0 => None,
                                1 => Some(ZscCollisionShape::Sphere),
                                2 => Some(ZscCollisionShape::AxisAlignedBoundingBox),
                                3 => Some(ZscCollisionShape::ObjectOrientedBoundingBox),
                                4 => Some(ZscCollisionShape::Polygon),
                                _ => bail!("Invalid ZscCollisionShape {}", bits & 0b111),
                            };
                            collision_flags = ZscCollisionFlags::from_bits(bits as u32 & !0b111)
                                .ok_or_else(|| {
                                    anyhow!("Invalid ZscCollisionFlags {}", bits as u32 & !0b111)
                                })?;
                        }
                        30 => {
                            animation_path = Some(VfsPathBuf::new(
                                &reader.read_fixed_length_string(size as usize)?,
                            ))
                        }
                        31 => reader.skip(2),
                        32 => reader.skip(2),
                        _ => bail!("Invalid ZscObjectPart property_id: {}", property_id),
                    }
                }

                object_parts.push(ZscObjectPart {
                    mesh_id,
                    material_id,
                    position: position.unwrap_or(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    rotation: rotation.unwrap_or(Vec4 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    }),
                    scale: scale.unwrap_or(Vec3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    }),
                    bone_index,
                    dummy_index,
                    parent,
                    collision_shape,
                    collision_flags,
                    animation_path,
                });
            }

            let object_effect_count = reader.read_u16()? as usize;
            let mut object_effects = Vec::with_capacity(object_effect_count);
            for _ in 0..object_effect_count {
                let effect_id = reader.read_u16()?;
                let effect_type = match reader.read_u16()? {
                    0 => ZscEffectType::Normal,
                    1 => ZscEffectType::DayNight,
                    2 => ZscEffectType::LightContainer,
                    invalid => ZscEffectType::Unknown(invalid),
                };

                let mut position = None;
                let mut rotation = None;
                let mut scale = None;
                let mut parent = None;

                loop {
                    let property_id = reader.read_u8()?;
                    if property_id == 0 {
                        break;
                    }
                    let _size = reader.read_u8()?;

                    match property_id {
                        1 => position = Some(reader.read_vector3_f32()?),
                        2 => {
                            let w = reader.read_f32()?;
                            let x = reader.read_f32()?;
                            let y = reader.read_f32()?;
                            let z = reader.read_f32()?;
                            rotation = Some(Vec4::<f32> { x, y, z, w });
                        }
                        3 => scale = Some(reader.read_vector3_f32()?),
                        7 => parent = {
                            let parent_id = reader.read_u16()?;
                            if parent_id == 0 {
                                None
                            } else {
                                Some(parent_id - 1)
                            }
                        },
                        _ => bail!("Invalid ZscObjectEffect property_id: {}", property_id),
                    }
                }

                object_effects.push(ZscObjectEffect {
                    effect_id,
                    effect_type,
                    position: position.unwrap_or(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    rotation: rotation.unwrap_or(Vec4 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        w: 0.0,
                    }),
                    scale: scale.unwrap_or(Vec3 {
                        x: 1.0,
                        y: 1.0,
                        z: 1.0,
                    }),
                    parent,
                });
            }

            reader.skip(4 * 3 * 2);

            objects.push(ZscObject {
                parts: object_parts,
                effects: object_effects,
            });
        }

        Ok(Self {
            meshes,
            materials,
            effects,
            objects,
        })
    }
}
