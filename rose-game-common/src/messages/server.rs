use bevy::math::{Vec2, Vec3};
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use rose_data::{
    AbilityType, AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemReference, MotionId, NpcId,
    QuestTriggerHash, SkillId, StackableItem, StatusEffectType, VehiclePartIndex, WorldTicks,
    ZoneId,
};

use crate::{
    components::{
        ActiveStatusEffect, BasicStatType, BasicStats, CharacterDeleteTime, CharacterInfo,
        CharacterUniqueId, DroppedItem, Equipment, ExperiencePoints, HealthPoints, Hotbar,
        HotbarSlot, Inventory, ItemSlot, Level, ManaPoints, Money, MoveMode, MoveSpeed, Npc,
        QuestState, SkillList, SkillPoints, SkillSlot, Stamina, StatPoints, Team, UnionMembership,
    },
    data::Damage,
    messages::{ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing},
};

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum ConnectionRequestError {
    #[error("Failed")]
    Failed,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid password")]
    InvalidPassword,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionResponse {
    pub packet_sequence_id: u32,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum LoginError {
    #[error("Login failed")]
    Failed,
    #[error("Invalid account")]
    InvalidAccount,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Already logged in")]
    AlreadyLoggedIn,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub server_list: Vec<(u32, String)>,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum ChannelListError {
    #[error("Invalid server id")]
    InvalidServerId(usize),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelList {
    pub server_id: usize,
    pub channels: Vec<(u8, String)>,
}

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum JoinServerError {
    #[error("Invalid server id")]
    InvalidServerId,
    #[error("Invalid channel id")]
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

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum CreateCharacterError {
    #[error("Failed")]
    Failed,

    #[error("Character name already exists")]
    AlreadyExists,

    #[error("Invalid value")]
    InvalidValue,

    #[error("No more free character slots")]
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
pub struct CharacterData {
    pub character_info: CharacterInfo,
    pub position: Vec3,
    pub zone_id: ZoneId,
    pub basic_stats: BasicStats,
    pub level: Level,
    pub equipment: Equipment,
    pub experience_points: ExperiencePoints,
    pub skill_list: SkillList,
    pub hotbar: Hotbar,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
    pub union_membership: UnionMembership,
    pub stamina: Stamina,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterDataItems {
    pub inventory: Inventory,
    pub equipment: Equipment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterDataQuest {
    pub quest_state: QuestState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinZoneResponse {
    pub entity_id: ClientEntityId,
    pub experience_points: ExperiencePoints,
    pub team: Team,
    pub health_points: HealthPoints,
    pub mana_points: ManaPoints,
    pub world_ticks: WorldTicks,
    pub craft_rate: i32,
    pub world_price_rate: i32,
    pub item_price_rate: i32,
    pub town_price_rate: i32,
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
    pub position: Vec3,
    pub remaining_time: Duration,
    pub owner_entity_id: Option<ClientEntityId>,
}

pub type ActiveStatusEffects = EnumMap<StatusEffectType, Option<ActiveStatusEffect>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandState {
    Stop,
    Emote,
    Move,
    Attack,
    Die,
    PickupItemDrop,
    CastSkillSelf,
    CastSkillTargetEntity,
    CastSkillTargetPosition,
    RunAway,
    Sit,
    PersonalStore,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnEntityCharacter {
    pub character_info: CharacterInfo,
    pub command: CommandState,
    pub destination: Option<Vec3>,
    pub entity_id: ClientEntityId,
    pub equipment: Equipment,
    pub health: HealthPoints,
    pub level: Level,
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub passive_attack_speed: i32,
    pub position: Vec3,
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
    pub position: Vec3,
    pub team: Team,
    pub health: HealthPoints,
    pub destination: Option<Vec3>,
    pub command: CommandState,
    pub target_entity_id: Option<ClientEntityId>,
    pub move_mode: MoveMode,
    pub status_effects: ActiveStatusEffects,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnEntityMonster {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub position: Vec3,
    pub team: Team,
    pub health: HealthPoints,
    pub destination: Option<Vec3>,
    pub command: CommandState,
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
    pub is_immediate: bool,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonalStoreTransactionStatus {
    Cancelled,
    SoldOut,
    NoMoreNeed, // Similar to SoldOut but when selling item to store
    BoughtFromStore,
    SoldToStore,
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
    pub target_position: Vec2,
    pub cast_motion_id: Option<MotionId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CastSkillTargetPosition {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_position: Vec2,
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

    pub fn get_client_entity_id(&self) -> Option<ClientEntityId> {
        match self {
            PartyMemberInfo::Online(info) => Some(info.entity_id),
            PartyMemberInfo::Offline(_) => None,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            PartyMemberInfo::Online(info) => &info.name,
            PartyMemberInfo::Offline(info) => &info.name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberList {
    pub item_sharing: PartyItemSharing,
    pub xp_sharing: PartyXpSharing,
    pub owner_character_id: CharacterUniqueId,
    pub members: Vec<PartyMemberInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartyMemberLeave {
    pub leaver_character_id: CharacterUniqueId,
    pub owner_character_id: CharacterUniqueId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSkillData {
    pub skill_slot: SkillSlot,
    pub skill_id: Option<SkillId>,
    pub expire_time: Option<Duration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CraftInsertGemError {
    NoSocket,
    SocketFull,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ConnectionResponse(Result<ConnectionResponse, ConnectionRequestError>),
    LoginResponse(Result<LoginResponse, LoginError>),
    ChannelList(Result<ChannelList, ChannelListError>),
    JoinServer(Result<JoinServerResponse, JoinServerError>),
    CharacterList(Vec<CharacterListItem>),
    CharacterListAppend(Vec<CharacterListItem>),
    CreateCharacter(Result<CreateCharacterResponse, CreateCharacterError>),
    DeleteCharacter(Result<DeleteCharacterResponse, DeleteCharacterError>),
    SelectCharacter(Result<JoinServerResponse, SelectCharacterError>),
    CharacterData(Box<CharacterData>),
    CharacterDataItems(Box<CharacterDataItems>),
    CharacterDataQuest(Box<CharacterDataQuest>),
    JoinZone(JoinZoneResponse),
    AttackEntity(AttackEntity),
    DamageEntity(DamageEntity),
    LocalChat(LocalChat),
    ShoutChat(ShoutChat),
    AnnounceChat(AnnounceChat),
    MoveEntity(MoveEntity),
    LevelUpEntity(ClientEntityId),
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
    UpdateItemLife {
        item_slot: ItemSlot,
        life: u16,
    },
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
    ClosePersonalStore(ClientEntityId),
    PersonalStoreItemList(PersonalStoreItemList),
    PersonalStoreTransaction {
        status: PersonalStoreTransactionStatus,
        store_entity_id: ClientEntityId,
        update_store: Vec<(usize, Option<Item>)>,
    },
    PersonalStoreTransactionUpdateInventory {
        money: Money,
        items: Vec<(ItemSlot, Option<Item>)>,
    },
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
    PartyCreate(ClientEntityId),
    PartyInvite(ClientEntityId),
    PartyAcceptCreate(ClientEntityId),
    PartyAcceptInvite(ClientEntityId),
    PartyRejectInvite(PartyRejectInviteReason, ClientEntityId),
    PartyChangeOwner(ClientEntityId),
    PartyDelete,
    PartyUpdateRules(PartyItemSharing, PartyXpSharing),
    PartyMemberList(PartyMemberList),
    PartyMemberLeave(PartyMemberLeave),
    PartyMemberDisconnect(CharacterUniqueId),
    PartyMemberKicked(CharacterUniqueId),
    PartyMemberUpdateInfo(PartyMemberInfoOnline),
    PartyMemberRewardItem {
        client_entity_id: ClientEntityId,
        item: Item,
    },
    ChangeNpcId(ClientEntityId, NpcId),
    SetHotbarSlot(usize, Option<HotbarSlot>),
    AdjustPosition(ClientEntityId, Vec3),
    UpdateSkillList(Vec<UpdateSkillData>),
    CraftInsertGem(Result<Vec<(ItemSlot, Option<Item>)>, CraftInsertGemError>),
    BankOpen,
    BankSetItems {
        items: Vec<(u8, Option<Item>)>,
    },
    BankUpdateItems {
        items: Vec<(u8, Option<Item>)>,
    },
    BankTransaction {
        inventory_item_slot: ItemSlot,
        inventory_item: Option<Item>,
        inventory_money: Option<Money>,
        bank_slot: usize,
        bank_item: Option<Item>,
    },
    RepairedItemUsingNpc {
        item_slot: ItemSlot,
        item: Item,
        updated_money: Money,
    },
}
