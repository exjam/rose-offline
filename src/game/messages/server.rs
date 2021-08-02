use std::time::Duration;

use nalgebra::Point2;

use crate::{
    data::{
        item::{EquipmentItem, Item},
        AbilityType, Damage, ItemReference, NpcId, QuestTriggerHash, SkillId, ZoneId,
    },
    game::components::{
        BasicStatType, CharacterInfo, ClientEntityId, Command, Destination, DroppedItem, Equipment,
        EquipmentIndex, ExperiencePoints, HealthPoints, ItemSlot, Level, ManaPoints, Money,
        MoveMode, MoveSpeed, Npc, NpcStandingDirection, Position, SkillPoints, SkillSlot,
        StatPoints, StatusEffects, Team,
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
pub enum PickupDroppedItemError {
    NotExist,
    NoPermission,
    InventoryFull,
}

#[derive(Clone)]
pub enum PickupDroppedItemContent {
    Item(ItemSlot, Item),
    Money(Money),
}

#[derive(Clone)]
pub struct PickupDroppedItemResult {
    pub item_entity_id: ClientEntityId,
    pub result: Result<PickupDroppedItemContent, PickupDroppedItemError>,
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
pub struct SpawnEntityCharacter {
    pub character_info: CharacterInfo,
    pub command: Command,
    pub destination: Option<Destination>,
    pub entity_id: ClientEntityId,
    pub equipment: Equipment,
    pub health: HealthPoints,
    pub level: Level,
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub passive_attack_speed: i32,
    pub position: Position,
    pub status_effects: StatusEffects,
    pub target_entity_id: Option<ClientEntityId>,
    pub team: Team,
    pub personal_store_info: Option<(i32, String)>,
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
    pub move_mode: MoveMode,
    pub status_effects: StatusEffects,
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
    pub move_mode: MoveMode,
    pub status_effects: StatusEffects,
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
    pub zone_id: ZoneId,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

#[derive(Clone)]
pub enum UpdateAbilityValue {
    RewardAdd(AbilityType, i32),
    RewardSet(AbilityType, i32),
}

#[derive(Clone)]
pub struct UpdateBasicStat {
    pub basic_stat_type: BasicStatType,
    pub value: i32,
}

#[derive(Clone)]
pub struct UpdateEquipment {
    pub entity_id: ClientEntityId,
    pub equipment_index: EquipmentIndex,
    pub item: Option<EquipmentItem>,
}

#[derive(Clone)]
pub struct UpdateInventory {
    pub is_reward: bool,
    pub items: Vec<(ItemSlot, Option<Item>)>,
}

#[derive(Clone)]
pub struct UpdateMoney {
    pub is_reward: bool,
    pub money: Money,
}

#[derive(Clone)]
pub struct UpdateStatusEffects {
    pub entity_id: ClientEntityId,
    pub status_effects: StatusEffects,
    pub updated_hp: Option<HealthPoints>,
    pub updated_mp: Option<ManaPoints>,
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
pub struct LogoutReply {
    pub result: Result<(), Duration>,
}

#[derive(Clone)]
pub struct QuestTriggerResult {
    pub success: bool,
    pub trigger_hash: QuestTriggerHash,
}

#[derive(Clone)]
pub struct QuestDeleteResult {
    pub success: bool,
    pub slot: usize,
    pub quest_id: usize,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum LearnSkillError {
    AlreadyLearnt,
    JobRequirement,
    SkillRequirement,
    AbilityRequirement,
    Full,
    InvalidSkillId,
    SkillPointRequirement,
}

#[derive(Clone)]
pub struct LearnSkillSuccess {
    pub skill_slot: SkillSlot,
    pub skill_id: SkillId,
    pub updated_skill_points: SkillPoints,
}

#[derive(Clone)]
pub struct OpenPersonalStore {
    pub entity_id: ClientEntityId,
    pub skin: i32,
    pub title: String,
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum PersonalStoreTransactionResult {
    Cancelled(PersonalStoreTransactionCancelled),
    SoldOut(PersonalStoreTransactionSoldOut),
    NoMoreNeed(PersonalStoreTransactionSoldOut),
    BoughtFromStore(PersonalStoreTransactionSuccess),
    SoldToStore(PersonalStoreTransactionSuccess),
}

#[derive(Clone)]
pub struct PersonalStoreTransactionCancelled {
    pub store_entity_id: ClientEntityId,
}

#[derive(Clone)]
pub struct PersonalStoreTransactionSoldOut {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub item: Option<Item>,
}

#[derive(Clone)]
pub struct PersonalStoreTransactionSuccess {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub store_slot_item: Option<Item>,
    pub money: Money,
    pub inventory_slot: ItemSlot,
    pub inventory_item: Option<Item>,
}

#[derive(Clone)]
pub struct PersonalStoreItemList {
    pub sell_items: Vec<(u8, Item, Money)>,
    pub buy_items: Vec<(u8, Item, Money)>,
}

#[derive(Clone)]
pub struct UseItem {
    pub entity_id: ClientEntityId,
    pub item: ItemReference,
    pub inventory_slot: ItemSlot,
}

#[derive(Clone)]
pub struct CastSkillSelf {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub npc_motion_id: Option<usize>,
}

#[derive(Clone)]
pub struct CastSkillTargetEntity {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_entity_id: ClientEntityId,
    pub target_distance: f32,
    pub target_position: Point2<f32>,
    pub npc_motion_id: Option<usize>,
}

#[derive(Clone)]
pub struct CastSkillTargetPosition {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_position: Point2<f32>,
    pub npc_motion_id: Option<usize>,
}

#[derive(Clone)]
pub struct ApplySkillEffect {
    pub entity_id: ClientEntityId,
    pub caster_entity_id: ClientEntityId,
    pub caster_intelligence: i32,
    pub skill_id: SkillId,
    pub effect_success: [bool; 2],
}

#[derive(Clone)]
pub enum CancelCastingSkillReason {
    NeedAbility,
    NeedTarget,
    InvalidTarget,
}

#[derive(Clone)]
pub enum ServerMessage {
    AttackEntity(AttackEntity),
    DamageEntity(DamageEntity),
    LocalChat(LocalChat),
    MoveEntity(MoveEntity),
    PickupDroppedItemResult(PickupDroppedItemResult),
    RemoveEntities(RemoveEntities),
    SpawnEntityCharacter(Box<SpawnEntityCharacter>),
    SpawnEntityDroppedItem(SpawnEntityDroppedItem),
    SpawnEntityMonster(SpawnEntityMonster),
    SpawnEntityNpc(SpawnEntityNpc),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    UpdateAbilityValue(UpdateAbilityValue),
    UpdateBasicStat(UpdateBasicStat),
    UpdateEquipment(UpdateEquipment),
    UpdateInventory(UpdateInventory),
    UpdateLevel(UpdateLevel),
    UpdateMoney(UpdateMoney),
    UpdateStatusEffects(UpdateStatusEffects),
    UpdateXpStamina(UpdateXpStamina),
    Whisper(Whisper),
    LogoutReply(LogoutReply),
    ReturnToCharacterSelect,
    QuestTriggerResult(QuestTriggerResult),
    QuestDeleteResult(QuestDeleteResult),
    LearnSkillResult(Result<LearnSkillSuccess, LearnSkillError>),
    RunNpcDeathTrigger(NpcId),
    OpenPersonalStore(OpenPersonalStore),
    PersonalStoreItemList(PersonalStoreItemList),
    PersonalStoreTransactionResult(PersonalStoreTransactionResult),
    UseItem(UseItem),
    CastSkillSelf(CastSkillSelf),
    CastSkillTargetEntity(CastSkillTargetEntity),
    CastSkillTargetPosition(CastSkillTargetPosition),
    StartCastingSkill(ClientEntityId),
    ApplySkillEffect(ApplySkillEffect),
    FinishCastingSkill(ClientEntityId, SkillId),
    CancelCastingSkill(ClientEntityId, CancelCastingSkillReason),
}
