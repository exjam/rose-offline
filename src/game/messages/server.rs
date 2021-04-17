#[derive(Clone)]
pub struct LocalChat {
    pub entity_id: u16,
    pub text: String,
}

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
pub struct RemoveEntities {
    pub entity_ids: Vec<u16>,
}

impl From<u16> for RemoveEntities {
    fn from(entity_id: u16) -> Self {
        Self { entity_ids: vec![entity_id] }
    }
}

#[derive(Clone)]
pub struct StopMoveEntity {
    pub entity_id: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub struct Whisper {
    pub from: String,
    pub text: String,
}

#[derive(Clone)]
pub struct Teleport {
    pub entity_id: u16,
    pub zone_no: u16,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

#[derive(Clone)]
pub enum ServerMessage {
    LocalChat(LocalChat),
    RemoveEntities(RemoveEntities),
    MoveEntity(MoveEntity),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    Whisper(Whisper),
}
