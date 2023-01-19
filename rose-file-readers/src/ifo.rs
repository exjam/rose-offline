use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::{
    reader::RoseFileReader,
    types::{Quat4, Vec2, Vec3},
    RoseFile,
};

#[derive(Debug)]
pub struct IfoObject {
    pub object_name: String,
    pub minimap_position: Vec2<u32>,
    pub object_type: u32,
    pub object_id: u32,
    pub warp_id: u16,
    pub event_id: u16,
    pub position: Vec3<f32>,
    pub rotation: Quat4<f32>,
    pub scale: Vec3<f32>,
}

fn read_object(reader: &mut RoseFileReader) -> anyhow::Result<IfoObject> {
    let object_name = reader.read_u8_length_string()?;
    let warp_id = reader.read_u16()?;
    let event_id = reader.read_u16()?;
    let object_type = reader.read_u32()?;
    let object_id = reader.read_u32()?;
    let minimap_pos_x = reader.read_u32()?;
    let minimap_pos_y = reader.read_u32()?;
    let rotation = reader.read_quat4_xyzw_f32()?;
    let position = reader.read_vector3_f32()?;
    let scale = reader.read_vector3_f32()?;

    Ok(IfoObject {
        object_name: String::from(object_name),
        warp_id,
        event_id,
        object_type,
        object_id,
        minimap_position: Vec2 {
            x: minimap_pos_x,
            y: minimap_pos_y,
        },
        rotation,
        position,
        scale,
    })
}

pub struct IfoMonsterSpawn {
    pub id: u32,
    pub count: u32,
}

pub struct IfoMonsterSpawnPoint {
    pub object: IfoObject,
    pub basic_spawns: Vec<IfoMonsterSpawn>,
    pub tactic_spawns: Vec<IfoMonsterSpawn>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
}

pub struct IfoEventObject {
    pub object: IfoObject,
    pub quest_trigger_name: String,
    pub script_function_name: String,
}

pub struct IfoNpc {
    pub object: IfoObject,
    pub ai_id: u32,
    pub quest_file_name: String,
}

pub struct IfoFile {
    pub monster_spawns: Vec<IfoMonsterSpawnPoint>,
    pub npcs: Vec<IfoNpc>,
    pub event_objects: Vec<IfoEventObject>,
    pub animated_objects: Vec<IfoObject>,
    pub collision_objects: Vec<IfoObject>,
    pub deco_objects: Vec<IfoObject>,
    pub cnst_objects: Vec<IfoObject>,
    pub water_size: f32,
    pub water_planes: Vec<(Vec3<f32>, Vec3<f32>)>,
    pub warps: Vec<IfoObject>,
}

#[derive(FromPrimitive)]
enum BlockType {
    MapInfo = 0,
    DecoObject = 1,
    Npc = 2,
    CnstObject = 3,
    Sound = 4,
    Effect = 5,
    AnimatedObject = 6,
    LegacyWater = 7,
    MonsterSpawn = 8,
    WaterPlanes = 9,
    Warp = 10,
    CollisionObject = 11,
    EventObject = 12,
}

#[derive(Default, Clone, Copy)]
pub struct IfoReadOptions {
    pub skip_monster_spawns: bool,
    pub skip_npcs: bool,
    pub skip_animated_objects: bool,
    pub skip_collision_objects: bool,
    pub skip_event_objects: bool,
    pub skip_cnst_objects: bool,
    pub skip_deco_objects: bool,
    pub skip_water_planes: bool,
    pub skip_warp_objects: bool,
}

impl RoseFile for IfoFile {
    type ReadOptions = IfoReadOptions;
    type WriteOptions = ();

    fn read(
        mut reader: RoseFileReader,
        read_options: &IfoReadOptions,
    ) -> Result<Self, anyhow::Error> {
        let mut monster_spawns = Vec::new();
        let mut npcs = Vec::new();
        let mut event_objects = Vec::new();
        let mut animated_objects = Vec::new();
        let mut collision_objects = Vec::new();
        let mut cnst_objects = Vec::new();
        let mut deco_objects = Vec::new();
        let mut water_size = 0.0;
        let mut water_planes = Vec::new();
        let mut warps = Vec::new();

        let block_count = reader.read_u32()?;
        for _ in 0..block_count {
            let block_type = reader.read_u32()?;
            let block_offset = reader.read_u32()?;
            let next_block_header_offset = reader.position();
            reader.set_position(block_offset as u64);

            match FromPrimitive::from_u32(block_type) {
                Some(BlockType::AnimatedObject) => {
                    if !read_options.skip_animated_objects {
                        let object_count = reader.read_u32()? as usize;
                        animated_objects.reserve_exact(object_count);

                        for _ in 0..object_count {
                            animated_objects.push(read_object(&mut reader)?);
                        }
                    }
                }
                Some(BlockType::CollisionObject) => {
                    if !read_options.skip_collision_objects {
                        let object_count = reader.read_u32()? as usize;
                        collision_objects.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let object = read_object(&mut reader)?;
                            collision_objects.push(object);
                        }
                    }
                }
                Some(BlockType::CnstObject) => {
                    if !read_options.skip_cnst_objects {
                        let object_count = reader.read_u32()? as usize;
                        cnst_objects.reserve_exact(object_count);

                        for _ in 0..object_count {
                            cnst_objects.push(read_object(&mut reader)?);
                        }
                    }
                }
                Some(BlockType::DecoObject) => {
                    if !read_options.skip_deco_objects {
                        let object_count = reader.read_u32()? as usize;
                        cnst_objects.reserve_exact(object_count);

                        for _ in 0..object_count {
                            deco_objects.push(read_object(&mut reader)?);
                        }
                    }
                }
                Some(BlockType::EventObject) => {
                    if !read_options.skip_event_objects {
                        let object_count = reader.read_u32()? as usize;
                        event_objects.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let object = read_object(&mut reader)?;
                            let quest_trigger_name = reader.read_u8_length_string()?;
                            let script_function_name = reader.read_u8_length_string()?;
                            event_objects.push(IfoEventObject {
                                object,
                                quest_trigger_name: String::from(quest_trigger_name),
                                script_function_name: String::from(script_function_name),
                            })
                        }
                    }
                }
                Some(BlockType::Npc) => {
                    if !read_options.skip_npcs {
                        let object_count = reader.read_u32()? as usize;
                        npcs.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let object = read_object(&mut reader)?;
                            let ai_id = reader.read_u32()?;
                            let quest_file_name = reader.read_u8_length_string()?;
                            npcs.push(IfoNpc {
                                object,
                                ai_id,
                                quest_file_name: String::from(quest_file_name),
                            });
                        }
                    }
                }
                Some(BlockType::MonsterSpawn) => {
                    if !read_options.skip_monster_spawns {
                        let object_count = reader.read_u32()? as usize;
                        monster_spawns.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let object = read_object(&mut reader)?;
                            let _spawn_name = reader.read_u8_length_string()?;

                            let basic_count = reader.read_u32()?;
                            let mut basic_spawns = Vec::with_capacity(basic_count as usize);
                            for _ in 0..basic_count {
                                let _monster_name = reader.read_u8_length_string()?;
                                let monster_id = reader.read_u32()?;
                                let monster_count = reader.read_u32()?;
                                basic_spawns.push(IfoMonsterSpawn {
                                    id: monster_id,
                                    count: monster_count,
                                });
                            }

                            let tactic_count = reader.read_u32()?;
                            let mut tactic_spawns = Vec::with_capacity(basic_count as usize);
                            for _ in 0..tactic_count {
                                let _monster_name = reader.read_u8_length_string()?;
                                let monster_id = reader.read_u32()?;
                                let monster_count = reader.read_u32()?;
                                tactic_spawns.push(IfoMonsterSpawn {
                                    id: monster_id,
                                    count: monster_count,
                                });
                            }

                            let interval = reader.read_u32()?;
                            let limit_count = reader.read_u32()?;
                            let range = reader.read_u32()?;
                            let tactic_points = reader.read_u32()?;
                            monster_spawns.push(IfoMonsterSpawnPoint {
                                object,
                                basic_spawns,
                                tactic_spawns,
                                interval,
                                limit_count,
                                range,
                                tactic_points,
                            });
                        }
                    }
                }
                Some(BlockType::WaterPlanes) => {
                    if !read_options.skip_water_planes {
                        water_size = reader.read_f32()?;

                        let object_count = reader.read_u32()? as usize;
                        water_planes.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let start = reader.read_vector3_f32()?;
                            let end = reader.read_vector3_f32()?;
                            water_planes.push((start, end));
                        }
                    }
                }
                Some(BlockType::Warp) => {
                    if !read_options.skip_warp_objects {
                        let object_count = reader.read_u32()? as usize;
                        warps.reserve_exact(object_count);

                        for _ in 0..object_count {
                            let object = read_object(&mut reader)?;
                            warps.push(object);
                        }
                    }
                }
                _ => {} // We do not need every block when reading for server
            }

            reader.set_position(next_block_header_offset);
        }

        Ok(IfoFile {
            monster_spawns,
            npcs,
            event_objects,
            animated_objects,
            collision_objects,
            deco_objects,
            cnst_objects,
            water_size,
            water_planes,
            warps,
        })
    }
}
