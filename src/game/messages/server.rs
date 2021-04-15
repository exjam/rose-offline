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
    MoveEntity(MoveEntity),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    Whisper(Whisper),
}
