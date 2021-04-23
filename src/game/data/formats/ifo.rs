use super::reader::{FileReader, ReadError};
use nalgebra::{Point2, Point3, Quaternion, Vector3};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug)]
pub enum IfoReadError {
    UnexpectedEof,
}

impl From<ReadError> for IfoReadError {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::UnexpectedEof => IfoReadError::UnexpectedEof,
        }
    }
}

#[derive(FromPrimitive)]
enum BlockType {
    MapInfo = 0,
    Object = 1,
    Npc = 2,
    Building = 3,
    Sound = 4,
    Effect = 5,
    Animation = 6,
    Water = 7,
    MonsterSpawn = 8,
    Ocean = 9,
    Warp = 10,
    CollisionObject = 11,
    EventObject = 12,
}

#[allow(dead_code)]
pub struct Object {
    pub object_name: String,
    pub minimap_position: Point2<u32>,
    pub object_type: u32,
    pub object_id: u32,
    pub warp_id: u16,
    pub event_id: u16,
    pub position: Point3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

fn read_object(reader: &mut FileReader) -> Result<Object, IfoReadError> {
    let object_name = reader.read_u8_length_string()?;
    let warp_id = reader.read_u16()?;
    let event_id = reader.read_u16()?;
    let object_type = reader.read_u32()?;
    let object_id = reader.read_u32()?;
    let minimap_pos_x = reader.read_u32()?;
    let minimap_pos_y = reader.read_u32()?;
    let rotation = reader.read_quaternion_f32()?;
    let position = reader.read_vector3_f32()?;
    let scale = reader.read_vector3_f32()?;

    Ok(Object {
        object_name: String::from(object_name),
        warp_id,
        event_id,
        object_type,
        object_id,
        minimap_position: Point2::new(minimap_pos_x, minimap_pos_y),
        rotation,
        position: Point3::from(position),
        scale,
    })
}

pub struct MonsterSpawn {
    pub id: u32,
    pub count: u32,
}

pub struct MonsterSpawnPoint {
    pub object: Object,
    pub basic_spawns: Vec<MonsterSpawn>,
    pub tactic_spawns: Vec<MonsterSpawn>,
    pub interval: u32,
    pub limit_count: u32,
    pub range: u32,
    pub tactic_points: u32,
}

pub struct EventObject {
    pub object: Object,
    pub function_name: String,
    pub file_name: String,
}

pub struct Npc {
    pub object: Object,
    pub ai_id: u32,
    pub quest_file_name: String,
}

pub struct IfoFile {
    pub monster_spawns: Vec<MonsterSpawnPoint>,
    pub npcs: Vec<Npc>,
    pub event_objects: Vec<EventObject>,
}

#[allow(dead_code)]
impl IfoFile {
    pub fn read(mut reader: FileReader) -> Result<Self, IfoReadError> {
        let mut monster_spawns = Vec::new();
        let mut npcs = Vec::new();
        let mut event_objects = Vec::new();

        let block_count = reader.read_u32()?;
        for _ in 0..block_count {
            let block_type = reader.read_u32()?;
            let block_offset = reader.read_u32()?;
            let next_block_header_offset = reader.position();
            reader.set_position(block_offset as u64);

            match FromPrimitive::from_u32(block_type) {
                Some(BlockType::EventObject) => {
                    let object_count = reader.read_u32()?;
                    for _ in 0..object_count {
                        let object = read_object(&mut reader)?;
                        let function_name = reader.read_u8_length_string()?;
                        let file_name = reader.read_u8_length_string()?;
                        event_objects.push(EventObject {
                            object,
                            function_name: String::from(function_name),
                            file_name: String::from(file_name),
                        })
                    }
                }
                Some(BlockType::Npc) => {
                    let object_count = reader.read_u32()?;
                    for _ in 0..object_count {
                        let object = read_object(&mut reader)?;
                        let ai_id = reader.read_u32()?;
                        let quest_file_name = reader.read_u8_length_string()?;
                        npcs.push(Npc {
                            object,
                            ai_id,
                            quest_file_name: String::from(quest_file_name),
                        });
                    }
                }
                Some(BlockType::MonsterSpawn) => {
                    let object_count = reader.read_u32()?;
                    for _ in 0..object_count {
                        let object = read_object(&mut reader)?;
                        let _spawn_name = reader.read_u8_length_string()?;

                        let basic_count = reader.read_u32()?;
                        let mut basic_spawns = Vec::with_capacity(basic_count as usize);
                        for _ in 0..basic_count {
                            let _monster_name = reader.read_u8_length_string()?;
                            let monster_id = reader.read_u32()?;
                            let monster_count = reader.read_u32()?;
                            basic_spawns.push(MonsterSpawn {
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
                            tactic_spawns.push(MonsterSpawn {
                                id: monster_id,
                                count: monster_count,
                            });
                        }

                        let interval = reader.read_u32()?;
                        let limit_count = reader.read_u32()?;
                        let range = reader.read_u32()?;
                        let tactic_points = reader.read_u32()?;
                        monster_spawns.push(MonsterSpawnPoint {
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
                _ => {}
            }

            reader.set_position(next_block_header_offset);
        }

        Ok(IfoFile {
            event_objects,
            monster_spawns,
            npcs,
        })
    }
}
