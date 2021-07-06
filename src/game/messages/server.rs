use std::time::Duration;

use crate::{
    data::{
        item::{EquipmentItem, Item},
        Damage,
    },
    game::components::{
        BasicStatType, ClientEntityId, Command, Destination, DroppedItem, EquipmentIndex,
        ExperiencePoints, HealthPoints, ItemSlot, Level, Npc, NpcStandingDirection, Position,
        SkillPoints, StatPoints, Team,
    },
};

#[derive(Clone)]
pub struct LocalChat {
    pub entity_id: ClientEntityId,
    pub text: String,
}

#[derive(Clone)]
pub struct AttackEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: ClientEntityId,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub struct MoveEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: Option<ClientEntityId>,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub struct RemoveEntities {
    pub entity_ids: Vec<ClientEntityId>,
}

impl From<ClientEntityId> for RemoveEntities {
    fn from(entity_id: ClientEntityId) -> Self {
        Self {
            entity_ids: vec![entity_id],
        }
    }
}

impl RemoveEntities {
    pub fn new(entity_ids: Vec<ClientEntityId>) -> Self {
        Self { entity_ids }
    }
}

#[derive(Clone)]
pub struct SpawnEntityDroppedItem {
    pub entity_id: ClientEntityId,
    pub dropped_item: DroppedItem,
    pub position: Position,
    pub remaining_time: Duration,
    pub owner_entity_id: Option<ClientEntityId>,
}

#[derive(Clone)]
pub struct SpawnEntityNpc {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub direction: NpcStandingDirection,
    pub position: Position,
    pub team: Team,
    pub health: HealthPoints,
    pub destination: Option<Destination>,
    pub command: Command,
    pub target_entity_id: Option<ClientEntityId>,
}

#[derive(Clone)]
pub struct SpawnEntityMonster {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub position: Position,
    pub team: Team,
    pub health: HealthPoints,
    pub destination: Option<Destination>,
    pub command: Command,
    pub target_entity_id: Option<ClientEntityId>,
}

#[derive(Clone)]
pub struct StopMoveEntity {
    pub entity_id: ClientEntityId,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone)]
pub struct DamageEntity {
    pub attacker_entity_id: ClientEntityId,
    pub defender_entity_id: ClientEntityId,
    pub damage: Damage,
    pub is_killed: bool,
}

#[derive(Clone)]
pub struct Whisper {
    pub from: String,
    pub text: String,
}

#[derive(Clone)]
pub struct Teleport {
    pub entity_id: ClientEntityId,
    pub zone_no: u16,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

#[derive(Clone)]
pub struct UpdateBasicStat {
    pub basic_stat_type: BasicStatType,
    pub value: u16,
}

#[derive(Clone)]
pub struct UpdateEquipment {
    pub entity_id: ClientEntityId,
    pub equipment_index: EquipmentIndex,
    pub item: Option<EquipmentItem>,
}

#[derive(Clone)]
pub struct UpdateInventory {
    pub items: Vec<(ItemSlot, Option<Item>)>,
}

#[derive(Clone)]
pub struct UpdateLevel {
    pub entity_id: ClientEntityId,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
}

#[derive(Clone)]
pub struct UpdateXpStamina {
    pub xp: u64,
    pub stamina: u32,
    pub source_entity_id: Option<ClientEntityId>,
}

#[derive(Clone)]
pub enum ServerMessage {
    LocalChat(LocalChat),
    SpawnEntityDroppedItem(SpawnEntityDroppedItem),
    SpawnEntityNpc(SpawnEntityNpc),
    SpawnEntityMonster(SpawnEntityMonster),
    RemoveEntities(RemoveEntities),
    AttackEntity(AttackEntity),
    DamageEntity(DamageEntity),
    MoveEntity(MoveEntity),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    Whisper(Whisper),
    UpdateBasicStat(UpdateBasicStat),
    UpdateEquipment(UpdateEquipment),
    UpdateInventory(UpdateInventory),
    UpdateLevel(UpdateLevel),
    UpdateXpStamina(UpdateXpStamina),
}
