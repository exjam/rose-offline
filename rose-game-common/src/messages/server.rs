use enum_map::EnumMap;
use nalgebra::Point2;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use rose_data::{
    AbilityType, AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemReference, MotionId, NpcId,
    QuestTriggerHash, SkillId, StackableItem, StatusEffectType, VehiclePartIndex, WorldTicks,
    ZoneId,
};

use crate::{
    components::{
        ActiveStatusEffect, BasicStatType, BasicStats, CharacterDeleteTime, CharacterInfo,
        CharacterUniqueId, ClientEntityId, Command, Destination, DroppedItem, Equipment,
        ExperiencePoints, HealthPoints, Hotbar, HotbarSlot, Inventory, ItemSlot, Level, ManaPoints,
        Money, MoveMode, MoveSpeed, Npc, Position, QuestState, SkillList, SkillPoints, SkillSlot,
        Stamina, StatPoints, Team, UnionMembership,
    },
    data::Damage,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConnectionRequestError {
    Failed,
    InvalidToken,
    InvalidPassword,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionResponse {
    pub packet_sequence_id: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LoginError {
    Failed,
    InvalidAccount,
    InvalidPassword,
    AlreadyLoggedIn,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub server_list: Vec<(u32, String)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChannelListError {
    InvalidServerId(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelList {
    pub server_id: usize,
    pub channels: Vec<(u8, String)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum JoinServerError {
    InvalidServerId,
    InvalidChannelId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinServerResponse {
    pub login_token: u32,
    pub packet_codec_seed: u32,
    pub ip: String,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterListItem {
    pub info: CharacterInfo,
    pub level: Level,
    pub delete_time: Option<CharacterDeleteTime>,
    pub equipment: Equipment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CreateCharacterError {
    Failed,
    AlreadyExists,
    InvalidValue,
    NoMoreSlots,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCharacterResponse {
    pub character_slot: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeleteCharacterError {
    Failed(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteCharacterResponse {
    pub name: String,
    pub delete_time: Option<CharacterDeleteTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SelectCharacterError {
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConnectionResponse {
    pub packet_sequence_id: u32,
    pub character_info: CharacterInfo,
    pub position: Position,
    pub equipment: Equipment,
    pub basic_stats: BasicStats,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub inventory: Inventory,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
    pub quest_state: QuestState,
    pub union_membership: UnionMembership,
    pub stamina: Stamina,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinZoneResponse {
    pub entity_id: ClientEntityId,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub team: Team,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub world_ticks: WorldTicks,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalChat {
    pub entity_id: ClientEntityId,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShoutChat {
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnnounceChat {
    pub name: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AttackEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: ClientEntityId,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: Option<ClientEntityId>,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
    pub move_mode: Option<MoveMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PickupItemDropError {
    NotExist,
    NoPermission,
    InventoryFull,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PickupItemDropContent {
    Item(ItemSlot, Item),
    Money(Money),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PickupItemDropResult {
    pub item_entity_id: ClientEntityId,
    pub result: Result<PickupItemDropContent, PickupItemDropError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnEntityItemDrop {
    pub entity_id: ClientEntityId,
    pub dropped_item: DroppedItem,
    pub position: Position,
    pub remaining_time: Duration,
    pub owner_entity_id: Option<ClientEntityId>,
}

pub type ActiveStatusEffects = EnumMap<StatusEffectType, Option<ActiveStatusEffect>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub status_effects: ActiveStatusEffects,
    pub target_entity_id: Option<ClientEntityId>,
    pub team: Team,
    pub personal_store_info: Option<(i32, String)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnEntityNpc {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub direction: f32,
    pub position: Position,
    pub team: Team,
    pub health: HealthPoints,
    pub destination: Option<Destination>,
    pub command: Command,
    pub target_entity_id: Option<ClientEntityId>,
    pub move_mode: MoveMode,
    pub status_effects: ActiveStatusEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub status_effects: ActiveStatusEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StopMoveEntity {
    pub entity_id: ClientEntityId,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DamageEntity {
    pub attacker_entity_id: ClientEntityId,
    pub defender_entity_id: ClientEntityId,
    pub damage: Damage,
    pub is_killed: bool,
    pub from_skill: Option<(SkillId, i32)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Whisper {
    pub from: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Teleport {
    pub entity_id: ClientEntityId,
    pub zone_id: ZoneId,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UpdateAbilityValue {
    RewardAdd(AbilityType, i32),
    RewardSet(AbilityType, i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateBasicStat {
    pub basic_stat_type: BasicStatType,
    pub value: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateEquipment {
    pub entity_id: ClientEntityId,
    pub equipment_index: EquipmentIndex,
    pub item: Option<EquipmentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateVehiclePart {
    pub entity_id: ClientEntityId,
    pub vehicle_part_index: VehiclePartIndex,
    pub item: Option<EquipmentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateStatusEffects {
    pub entity_id: ClientEntityId,
    pub status_effects: ActiveStatusEffects,
    pub updated_hp: Option<HealthPoints>,
    pub updated_mp: Option<ManaPoints>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSpeed {
    pub entity_id: ClientEntityId,
    pub run_speed: i32,
    pub passive_attack_speed: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateLevel {
    pub entity_id: ClientEntityId,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateXpStamina {
    pub xp: u64,
    pub stamina: u32,
    pub source_entity_id: Option<ClientEntityId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogoutReply {
    pub result: Result<(), Duration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestTriggerResult {
    pub success: bool,
    pub trigger_hash: QuestTriggerHash,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuestDeleteResult {
    pub success: bool,
    pub slot: usize,
    pub quest_id: usize,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LearnSkillError {
    AlreadyLearnt,
    JobRequirement,
    SkillRequirement,
    AbilityRequirement,
    Full,
    InvalidSkillId,
    SkillPointRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearnSkillSuccess {
    pub skill_slot: SkillSlot,
    pub skill_id: Option<SkillId>,
    pub updated_skill_points: SkillPoints,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum LevelUpSkillError {
    Failed,
    SkillPointRequirement,
    AbilityRequirement,
    JobRequirement,
    SkillRequirement,
    MoneyRequirement,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevelUpSkillResult {
    pub result: Result<(SkillSlot, SkillId), LevelUpSkillError>,
    pub updated_skill_points: SkillPoints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenPersonalStore {
    pub entity_id: ClientEntityId,
    pub skin: i32,
    pub title: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonalStoreTransactionResult {
    Cancelled(PersonalStoreTransactionCancelled),
    SoldOut(PersonalStoreTransactionSoldOut),
    NoMoreNeed(PersonalStoreTransactionSoldOut),
    BoughtFromStore(PersonalStoreTransactionSuccess),
    SoldToStore(PersonalStoreTransactionSuccess),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalStoreTransactionCancelled {
    pub store_entity_id: ClientEntityId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalStoreTransactionSoldOut {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub item: Option<Item>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalStoreTransactionSuccess {
    pub store_entity_id: ClientEntityId,
    pub store_slot_index: usize,
    pub store_slot_item: Option<Item>,
    pub money: Money,
    pub inventory_slot: ItemSlot,
    pub inventory_item: Option<Item>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonalStoreItemList {
    pub sell_items: Vec<(u8, Item, Money)>,
    pub buy_items: Vec<(u8, Item, Money)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UseItem {
    pub entity_id: ClientEntityId,
    pub item: ItemReference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UseInventoryItem {
    pub entity_id: ClientEntityId,
    pub item: ItemReference,
    pub inventory_slot: ItemSlot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CastSkillSelf {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub cast_motion_id: Option<MotionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CastSkillTargetEntity {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_entity_id: ClientEntityId,
    pub target_distance: f32,
    pub target_position: Point2<f32>,
    pub cast_motion_id: Option<MotionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CastSkillTargetPosition {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_position: Point2<f32>,
    pub cast_motion_id: Option<MotionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplySkillEffect {
    pub entity_id: ClientEntityId,
    pub caster_entity_id: ClientEntityId,
    pub caster_intelligence: i32,
    pub skill_id: SkillId,
    pub effect_success: [bool; 2],
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CancelCastingSkillReason {
    NeedAbility,
    NeedTarget,
    InvalidTarget,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NpcStoreTransactionError {
    PriceDifference,
    NpcNotFound,
    NpcTooFarAway,
    NotEnoughMoney,
    NotSameUnion,
    NotEnoughUnionPoints,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveToggle {
    pub entity_id: ClientEntityId,
    pub move_mode: MoveMode,
    pub run_speed: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UseEmote {
    pub entity_id: ClientEntityId,
    pub motion_id: MotionId,
    pub is_stop: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartyRequest {
    Create(ClientEntityId),
    Invite(ClientEntityId),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartyReply {
    AcceptCreate(ClientEntityId),
    AcceptInvite(ClientEntityId),
    RejectInvite(ClientEntityId),
    DeleteParty,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberInfoOnline {
    pub character_id: CharacterUniqueId,
    pub name: String,
    pub entity_id: ClientEntityId,
    pub health_points: HealthPoints,
    pub status_effects: ActiveStatusEffects,
    pub max_health: i32,
    pub concentration: i32,
    pub health_recovery: i32,
    pub mana_recovery: i32,
    pub stamina: Stamina,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberInfoOffline {
    pub character_id: CharacterUniqueId,
    pub name: String,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PartyMemberInfo {
    Online(PartyMemberInfoOnline),
    Offline(PartyMemberInfoOffline),
}

impl PartyMemberInfo {
    pub fn get_character_id(&self) -> CharacterUniqueId {
        match self {
            PartyMemberInfo::Online(info) => info.character_id,
            PartyMemberInfo::Offline(info) => info.character_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberList {
    pub owner_character_id: CharacterUniqueId,
    pub members: Vec<PartyMemberInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberLeave {
    pub leaver_character_id: CharacterUniqueId,
    pub owner_character_id: CharacterUniqueId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ConnectionResponse(Result<ConnectionResponse, ConnectionRequestError>),
    LoginResponse(Result<LoginResponse, LoginError>),
    ChannelList(Result<ChannelList, ChannelListError>),
    JoinServer(Result<JoinServerResponse, JoinServerError>),
    CharacterList(Vec<CharacterListItem>),
    CreateCharacter(Result<CreateCharacterResponse, CreateCharacterError>),
    DeleteCharacter(Result<DeleteCharacterResponse, DeleteCharacterError>),
    SelectCharacter(Result<JoinServerResponse, SelectCharacterError>),
    GameConnectionResponse(Result<Box<GameConnectionResponse>, ConnectionRequestError>),
    JoinZone(JoinZoneResponse),
    AttackEntity(AttackEntity),
    DamageEntity(DamageEntity),
    LocalChat(LocalChat),
    ShoutChat(ShoutChat),
    AnnounceChat(AnnounceChat),
    MoveEntity(MoveEntity),
    PickupItemDropResult(PickupItemDropResult),
    RemoveEntities(RemoveEntities),
    SpawnEntityCharacter(Box<SpawnEntityCharacter>),
    SpawnEntityItemDrop(SpawnEntityItemDrop),
    SpawnEntityMonster(SpawnEntityMonster),
    SpawnEntityNpc(SpawnEntityNpc),
    StopMoveEntity(StopMoveEntity),
    Teleport(Teleport),
    UpdateAbilityValue(UpdateAbilityValue),
    UpdateAmmo(ClientEntityId, AmmoIndex, Option<StackableItem>),
    UpdateBasicStat(UpdateBasicStat),
    UpdateEquipment(UpdateEquipment),
    UpdateVehiclePart(UpdateVehiclePart),
    UpdateInventory(Vec<(ItemSlot, Option<Item>)>, Option<Money>),
    UpdateLevel(UpdateLevel),
    UpdateMoney(Money),
    UpdateStatusEffects(UpdateStatusEffects),
    UpdateSpeed(UpdateSpeed),
    UpdateXpStamina(UpdateXpStamina),
    RewardItems(Vec<(ItemSlot, Option<Item>)>),
    RewardMoney(Money),
    Whisper(Whisper),
    LogoutReply(LogoutReply),
    ReturnToCharacterSelect,
    QuestTriggerResult(QuestTriggerResult),
    QuestDeleteResult(QuestDeleteResult),
    LearnSkillResult(Result<LearnSkillSuccess, LearnSkillError>),
    LevelUpSkillResult(LevelUpSkillResult),
    RunNpcDeathTrigger(NpcId),
    OpenPersonalStore(OpenPersonalStore),
    PersonalStoreItemList(PersonalStoreItemList),
    PersonalStoreTransactionResult(PersonalStoreTransactionResult),
    UseItem(UseItem),
    UseInventoryItem(UseInventoryItem),
    CastSkillSelf(CastSkillSelf),
    CastSkillTargetEntity(CastSkillTargetEntity),
    CastSkillTargetPosition(CastSkillTargetPosition),
    StartCastingSkill(ClientEntityId),
    ApplySkillEffect(ApplySkillEffect),
    FinishCastingSkill(ClientEntityId, SkillId),
    CancelCastingSkill(ClientEntityId, CancelCastingSkillReason),
    NpcStoreTransactionError(NpcStoreTransactionError),
    MoveToggle(MoveToggle),
    SitToggle(ClientEntityId),
    UseEmote(UseEmote),
    PartyRequest(PartyRequest),
    PartyReply(PartyReply),
    PartyMemberList(PartyMemberList),
    PartyMemberLeave(PartyMemberLeave),
    PartyMemberDisconnect(CharacterUniqueId),
    PartyMemberKicked(CharacterUniqueId),
    PartyMemberUpdateInfo(PartyMemberInfoOnline),
    PartyChangeOwner(ClientEntityId),
    ChangeNpcId(ClientEntityId, NpcId),
    SetHotbarSlot(usize, Option<HotbarSlot>),
}
