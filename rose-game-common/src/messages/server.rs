use bevy::math::{Vec2, Vec3};
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, time::Duration};
use thiserror::Error;

use rose_data::{
    AbilityType, AmmoIndex, ClanMemberPosition, EquipmentIndex, EquipmentItem, Item, ItemReference,
    MotionId, NpcId, QuestTriggerHash, SkillId, StackableItem, StatusEffectType, VehiclePartIndex,
    WorldTicks, ZoneId,
};

use crate::{
    components::{
        ActiveStatusEffect, BasicStatType, BasicStats, CharacterDeleteTime, CharacterInfo,
        CharacterUniqueId, ClanLevel, ClanMark, ClanPoints, ClanUniqueId, DroppedItem, Equipment,
        ExperiencePoints, HealthPoints, Hotbar, HotbarSlot, Inventory, ItemSlot, Level, ManaPoints,
        Money, MoveMode, MoveSpeed, Npc, QuestState, SkillList, SkillPoints, SkillSlot, Stamina,
        StatPoints, Team, UnionMembership,
    },
    data::Damage,
    messages::{ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing},
};

#[derive(Copy, Clone, Debug, Error, Serialize, Deserialize)]
pub enum ConnectionRequestError {
    #[error("Failed")]
    Failed,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid password")]
    InvalidPassword,
}

#[derive(Copy, Clone, Debug, Error, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, Error, Serialize, Deserialize)]
pub enum ChannelListError {
    #[error("Invalid server id")]
    InvalidServerId { server_id: usize },
}

#[derive(Copy, Clone, Debug, Error, Serialize, Deserialize)]
pub enum JoinServerError {
    #[error("Invalid server id")]
    InvalidServerId,
    #[error("Invalid channel id")]
    InvalidChannelId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterListItem {
    pub info: CharacterInfo,
    pub level: Level,
    pub delete_time: Option<CharacterDeleteTime>,
    pub equipment: Equipment,
}

#[derive(Copy, Clone, Debug, Error, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PickupItemDropError {
    NotExist,
    NoPermission,
    InventoryFull,
}

pub type ActiveStatusEffects = EnumMap<StatusEffectType, Option<ActiveStatusEffect>>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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
pub struct CharacterClanMembership {
    pub clan_unique_id: ClanUniqueId,
    pub mark: ClanMark,
    pub level: ClanLevel,
    pub name: String,
    pub position: ClanMemberPosition,
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
    pub clan_membership: Option<CharacterClanMembership>,
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
pub enum PersonalStoreTransactionStatus {
    Cancelled,
    SoldOut,
    NoMoreNeed, // Similar to SoldOut but when selling item to store
    BoughtFromStore,
    SoldToStore,
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

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClanCreateError {
    Failed,
    NameExists,
    NoPermission,
    UnmetCondition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClanMemberInfo {
    pub name: String,
    pub position: ClanMemberPosition,
    pub contribution: ClanPoints,
    pub channel_id: Option<NonZeroUsize>,
    pub level: Level,
    pub job: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ConnectionRequestSuccess {
        packet_sequence_id: u32,
    },
    ConnectionRequestError {
        error: ConnectionRequestError,
    },
    LoginSuccess {
        server_list: Vec<(u32, String)>,
    },
    LoginError {
        error: LoginError,
    },
    ChannelList {
        server_id: usize,
        channels: Vec<(u8, String)>,
    },
    ChannelListError {
        error: ChannelListError,
    },
    JoinServerSuccess {
        login_token: u32,
        packet_codec_seed: u32,
        ip: String,
        port: u16,
    },
    JoinServerError {
        error: JoinServerError,
    },
    CharacterList {
        character_list: Vec<CharacterListItem>,
    },
    CharacterListAppend {
        character_list: Vec<CharacterListItem>,
    },
    CreateCharacterSuccess {
        character_slot: usize,
    },
    CreateCharacterError {
        error: CreateCharacterError,
    },
    DeleteCharacterStart {
        name: String,
        delete_time: CharacterDeleteTime,
    },
    DeleteCharacterCancel {
        name: String,
    },
    DeleteCharacterError {
        name: String,
    },
    SelectCharacterSuccess {
        login_token: u32,
        packet_codec_seed: u32,
        ip: String,
        port: u16,
    },
    SelectCharacterError,
    CharacterData {
        data: Box<CharacterData>,
    },
    CharacterDataItems {
        data: Box<CharacterDataItems>,
    },
    CharacterDataQuest {
        quest_state: Box<QuestState>,
    },
    JoinZone {
        entity_id: ClientEntityId,
        experience_points: ExperiencePoints,
        team: Team,
        health_points: HealthPoints,
        mana_points: ManaPoints,
        world_ticks: WorldTicks,
        craft_rate: i32,
        world_price_rate: i32,
        item_price_rate: i32,
        town_price_rate: i32,
    },
    AttackEntity {
        entity_id: ClientEntityId,
        target_entity_id: ClientEntityId,
        distance: u16,
        x: f32,
        y: f32,
        z: u16,
    },
    DamageEntity {
        attacker_entity_id: ClientEntityId,
        defender_entity_id: ClientEntityId,
        damage: Damage,
        is_killed: bool,
        is_immediate: bool,
        from_skill: Option<(SkillId, i32)>,
    },
    LocalChat {
        entity_id: ClientEntityId,
        text: String,
    },
    ShoutChat {
        name: String,
        text: String,
    },
    AnnounceChat {
        name: Option<String>,
        text: String,
    },
    MoveEntity {
        entity_id: ClientEntityId,
        target_entity_id: Option<ClientEntityId>,
        distance: u16,
        x: f32,
        y: f32,
        z: u16,
        move_mode: Option<MoveMode>,
    },
    LevelUpEntity {
        entity_id: ClientEntityId,
    },
    PickupDropItem {
        drop_entity_id: ClientEntityId,
        item_slot: ItemSlot,
        item: Item,
    },
    PickupDropMoney {
        drop_entity_id: ClientEntityId,
        money: Money,
    },
    PickupDropError {
        drop_entity_id: ClientEntityId,
        error: PickupItemDropError,
    },
    RemoveEntities {
        entity_ids: Vec<ClientEntityId>,
    },
    SpawnEntityCharacter {
        data: Box<SpawnEntityCharacter>,
    },
    SpawnEntityItemDrop {
        entity_id: ClientEntityId,
        dropped_item: DroppedItem,
        position: Vec3,
        remaining_time: Duration,
        owner_entity_id: Option<ClientEntityId>,
    },
    SpawnEntityMonster {
        entity_id: ClientEntityId,
        npc: Npc,
        position: Vec3,
        team: Team,
        health: HealthPoints,
        destination: Option<Vec3>,
        command: CommandState,
        target_entity_id: Option<ClientEntityId>,
        move_mode: MoveMode,
        status_effects: ActiveStatusEffects,
    },
    SpawnEntityNpc {
        entity_id: ClientEntityId,
        npc: Npc,
        direction: f32,
        position: Vec3,
        team: Team,
        health: HealthPoints,
        destination: Option<Vec3>,
        command: CommandState,
        target_entity_id: Option<ClientEntityId>,
        move_mode: MoveMode,
        status_effects: ActiveStatusEffects,
    },
    StopMoveEntity {
        entity_id: ClientEntityId,
        x: f32,
        y: f32,
        z: u16,
    },
    Teleport {
        entity_id: ClientEntityId,
        zone_id: ZoneId,
        x: f32,
        y: f32,
        run_mode: u8,
        ride_mode: u8,
    },
    UpdateAbilityValueAdd {
        ability_type: AbilityType,
        value: i32,
    },
    UpdateAbilityValueSet {
        ability_type: AbilityType,
        value: i32,
    },
    UpdateBasicStat {
        basic_stat_type: BasicStatType,
        value: i32,
    },
    UpdateAmmo {
        entity_id: ClientEntityId,
        ammo_index: AmmoIndex,
        item: Option<StackableItem>,
    },
    UpdateEquipment {
        entity_id: ClientEntityId,
        equipment_index: EquipmentIndex,
        item: Option<EquipmentItem>,
    },
    UpdateVehiclePart {
        entity_id: ClientEntityId,
        vehicle_part_index: VehiclePartIndex,
        item: Option<EquipmentItem>,
    },
    UpdateInventory {
        items: Vec<(ItemSlot, Option<Item>)>,
        money: Option<Money>,
    },
    UpdateLevel {
        entity_id: ClientEntityId,
        level: Level,
        experience_points: ExperiencePoints,
        stat_points: StatPoints,
        skill_points: SkillPoints,
    },
    UpdateMoney {
        money: Money,
    },
    UpdateStatusEffects {
        entity_id: ClientEntityId,
        status_effects: ActiveStatusEffects,
        updated_values: Vec<i32>,
    },
    UpdateSpeed {
        entity_id: ClientEntityId,
        run_speed: i32,
        passive_attack_speed: i32,
    },
    UpdateXpStamina {
        xp: u64,
        stamina: u32,
        source_entity_id: Option<ClientEntityId>,
    },
    UpdateItemLife {
        item_slot: ItemSlot,
        life: u16,
    },
    RewardItems {
        items: Vec<(ItemSlot, Option<Item>)>,
    },
    RewardMoney {
        money: Money,
    },
    Whisper {
        from: String,
        text: String,
    },
    LogoutSuccess,
    LogoutFailed {
        wait_duration: Duration,
    },
    ReturnToCharacterSelect,
    QuestTriggerResult {
        trigger_hash: QuestTriggerHash,
        success: bool,
    },
    QuestDeleteResult {
        success: bool,
        slot: usize,
        quest_id: usize,
    },
    LearnSkillSuccess {
        skill_slot: SkillSlot,
        skill_id: Option<SkillId>,
        updated_skill_points: SkillPoints,
    },
    LearnSkillError {
        error: LearnSkillError,
    },
    LevelUpSkillSuccess {
        skill_slot: SkillSlot,
        skill_id: SkillId,
        skill_points: SkillPoints,
    },
    LevelUpSkillError {
        error: LevelUpSkillError,
        skill_points: SkillPoints,
    },
    RunNpcDeathTrigger {
        npc_id: NpcId,
    },
    OpenPersonalStore {
        entity_id: ClientEntityId,
        skin: i32,
        title: String,
    },
    ClosePersonalStore {
        entity_id: ClientEntityId,
    },
    PersonalStoreItemList {
        sell_items: Vec<(u8, Item, Money)>,
        buy_items: Vec<(u8, Item, Money)>,
    },
    PersonalStoreTransaction {
        status: PersonalStoreTransactionStatus,
        store_entity_id: ClientEntityId,
        update_store: Vec<(usize, Option<Item>)>,
    },
    PersonalStoreTransactionUpdateInventory {
        money: Money,
        items: Vec<(ItemSlot, Option<Item>)>,
    },
    UseItem {
        entity_id: ClientEntityId,
        item: ItemReference,
    },
    UseInventoryItem {
        entity_id: ClientEntityId,
        item: ItemReference,
        inventory_slot: ItemSlot,
    },
    CastSkillSelf {
        entity_id: ClientEntityId,
        skill_id: SkillId,
        cast_motion_id: Option<MotionId>,
    },
    CastSkillTargetEntity {
        entity_id: ClientEntityId,
        skill_id: SkillId,
        target_entity_id: ClientEntityId,
        target_distance: f32,
        target_position: Vec2,
        cast_motion_id: Option<MotionId>,
    },
    CastSkillTargetPosition {
        entity_id: ClientEntityId,
        skill_id: SkillId,
        target_position: Vec2,
        cast_motion_id: Option<MotionId>,
    },
    StartCastingSkill {
        entity_id: ClientEntityId,
    },
    ApplySkillEffect {
        entity_id: ClientEntityId,
        caster_entity_id: ClientEntityId,
        caster_intelligence: i32,
        skill_id: SkillId,
        effect_success: [bool; 2],
    },
    FinishCastingSkill {
        entity_id: ClientEntityId,
        skill_id: SkillId,
    },
    CancelCastingSkill {
        entity_id: ClientEntityId,
        reason: CancelCastingSkillReason,
    },
    NpcStoreTransactionError {
        error: NpcStoreTransactionError,
    },
    MoveToggle {
        entity_id: ClientEntityId,
        move_mode: MoveMode,
        run_speed: Option<i32>,
    },
    SitToggle {
        entity_id: ClientEntityId,
    },
    UseEmote {
        entity_id: ClientEntityId,
        motion_id: MotionId,
        is_stop: bool,
    },
    PartyCreate {
        entity_id: ClientEntityId,
    },
    PartyInvite {
        entity_id: ClientEntityId,
    },
    PartyAcceptCreate {
        entity_id: ClientEntityId,
    },
    PartyAcceptInvite {
        entity_id: ClientEntityId,
    },
    PartyRejectInvite {
        reason: PartyRejectInviteReason,
        entity_id: ClientEntityId,
    },
    PartyChangeOwner {
        entity_id: ClientEntityId,
    },
    PartyDelete,
    PartyUpdateRules {
        item_sharing: PartyItemSharing,
        xp_sharing: PartyXpSharing,
    },
    PartyMemberList {
        item_sharing: PartyItemSharing,
        xp_sharing: PartyXpSharing,
        owner_character_id: CharacterUniqueId,
        members: Vec<PartyMemberInfo>,
    },
    PartyMemberLeave {
        leaver_character_id: CharacterUniqueId,
        owner_character_id: CharacterUniqueId,
    },
    PartyMemberDisconnect {
        character_id: CharacterUniqueId,
    },
    PartyMemberKicked {
        character_id: CharacterUniqueId,
    },
    PartyMemberUpdateInfo {
        member_info: PartyMemberInfoOnline,
    },
    PartyMemberRewardItem {
        client_entity_id: ClientEntityId,
        item: Item,
    },
    ChangeNpcId {
        entity_id: ClientEntityId,
        npc_id: NpcId,
    },
    SetHotbarSlot {
        slot_index: usize,
        slot: Option<HotbarSlot>,
    },
    AdjustPosition {
        entity_id: ClientEntityId,
        position: Vec3,
    },
    UpdateSkillList {
        skill_list: Vec<UpdateSkillData>,
    },
    CraftInsertGem {
        update_items: Vec<(ItemSlot, Option<Item>)>,
    },
    CraftInsertGemError {
        error: CraftInsertGemError,
    },
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
    ClanInfo {
        id: ClanUniqueId,
        mark: ClanMark,
        level: ClanLevel,
        points: ClanPoints,
        money: Money,
        name: String,
        description: String,
        position: ClanMemberPosition,
        contribution: ClanPoints,
        skills: Vec<SkillId>,
    },
    ClanUpdateInfo {
        id: ClanUniqueId,
        mark: ClanMark,
        level: ClanLevel,
        points: ClanPoints,
        money: Money,
        skills: Vec<SkillId>,
    },
    CharacterUpdateClan {
        client_entity_id: ClientEntityId,
        id: ClanUniqueId,
        name: String,
        mark: ClanMark,
        level: ClanLevel,
        position: ClanMemberPosition,
    },
    ClanMemberConnected {
        name: String,
        channel_id: NonZeroUsize,
    },
    ClanMemberDisconnected {
        name: String,
    },
    ClanCreateError {
        error: ClanCreateError,
    },
    ClanMemberList {
        members: Vec<ClanMemberInfo>,
    },
}
