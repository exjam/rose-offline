#[derive(Clone)]
pub struct MoveEntity {
    pub entity_id: u16,
    pub target_entity_id: u16,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub enum ServerMessage {
    MoveEntity(MoveEntity),
}
