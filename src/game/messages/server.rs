use crate::game::{
    components::{EquipmentIndex, ItemSlot, Npc, NpcStandingDirection, Position, Team},
    data::items::{EquipmentItem, Item},
};

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
        Self {
            entity_ids: vec![entity_id],
        }
    }
}

impl RemoveEntities {
    pub fn new(entity_ids: Vec<u16>) -> Self {
        Self { entity_ids }
    }
}

#[derive(Clone)]
pub struct SpawnEntityNpc {
    pub entity_id: u16,
    pub npc: Npc,
    pub direction: NpcStandingDirection,
    pub position: Position,
    pub team: Team,
}

#[derive(Clone)]
pub struct SpawnEntityMonster {
    pub entity_id: u16,
    pub npc: Npc,
    pub position: Position,
    pub team: Team,
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
pub struct UpdateInventory {
    pub items: Vec<(ItemSlot, Option<Item>)>,
}

#[derive(Clone)]
pub struct UpdateEquipment {
    pub entity_id: u16,
    pub equipment_index: EquipmentIndex,
    pub item: Option<EquipmentItem>,
}

#[derive(Clone)]
pub enum ServerMessage {
    LocalChat(LocalChat),
    SpawnEntityNpc(SpawnEntityNpc),
    SpawnEntityMonster(SpawnEntityMonster),
    RemoveEntities(RemoveEntities),
    MoveEntity(MoveEntity),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    Whisper(Whisper),
    UpdateInventory(UpdateInventory),
    UpdateEquipment(UpdateEquipment),
}
