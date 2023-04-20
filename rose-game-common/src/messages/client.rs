use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        BasicStatType, CharacterGender, CharacterUniqueId, ClanMark, HotbarSlot, ItemSlot, Level,
        SkillSlot,
    },
    data::Password,
    messages::{ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing},
};
use rose_data::{
    AmmoIndex, EquipmentIndex, Item, MotionId, QuestTriggerHash, VehiclePartIndex, WarpGateId,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NpcStoreBuyItem {
    pub tab_index: usize,
    pub item_index: usize,
    pub quantity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    ConnectionRequest {
        login_token: u32,
        password: Password,
    },
    LoginRequest {
        username: String,
        password: Password,
    },
    GetChannelList {
        server_id: usize,
    },
    JoinServer {
        server_id: usize,
        channel_id: usize,
    },
    GetCharacterList,
    CreateCharacter {
        gender: CharacterGender,
        hair: i32,
        face: i32,
        name: String,
        start_point: i32,

        // irose
        birth_stone: i32,

        // narose667
        hair_color: i32,
        weapon_type: i32,
    },
    DeleteCharacter {
        slot: u8,
        name: String,
        is_delete: bool,
    },
    SelectCharacter {
        slot: u8,
        name: String,
    },
    GameConnectionRequest {
        login_token: u32,
        password: Password,
    },
    JoinZoneRequest,
    Chat {
        text: String,
    },
    Move {
        target_entity_id: Option<ClientEntityId>,
        x: f32,
        y: f32,
        z: u16,
    },
    MoveCollision {
        position: Vec3,
    },
    Attack {
        target_entity_id: ClientEntityId,
    },
    SetHotbarSlot {
        slot_index: usize,
        slot: Option<HotbarSlot>,
    },
    ChangeAmmo {
        ammo_index: AmmoIndex,
        item_slot: Option<ItemSlot>,
    },
    ChangeEquipment {
        equipment_index: EquipmentIndex,
        item_slot: Option<ItemSlot>,
    },
    ChangeVehiclePart {
        vehicle_part_index: VehiclePartIndex,
        item_slot: Option<ItemSlot>,
    },
    IncreaseBasicStat {
        basic_stat_type: BasicStatType,
    },
    PickupItemDrop {
        target_entity_id: ClientEntityId,
    },
    Logout,
    ReturnToCharacterSelect,
    ReviveCurrentZone,
    ReviveSaveZone,
    SetReviveSaveZone,
    QuestTrigger {
        trigger: QuestTriggerHash,
    },
    QuestDelete {
        slot: usize,
        quest_id: usize,
    },
    PersonalStoreListItems {
        store_entity_id: ClientEntityId,
    },
    PersonalStoreBuyItem {
        store_entity_id: ClientEntityId,
        store_slot_index: usize,
        buy_item: Item,
    },
    DropItem {
        item_slot: ItemSlot,
        quantity: usize,
    },
    DropMoney {
        quantity: usize,
    },
    UseItem {
        item_slot: ItemSlot,
        target_entity_id: Option<ClientEntityId>,
    },
    LevelUpSkill {
        skill_slot: SkillSlot,
    },
    CastSkillSelf {
        skill_slot: SkillSlot,
    },
    CastSkillTargetEntity {
        skill_slot: SkillSlot,
        target_entity_id: ClientEntityId,
    },
    CastSkillTargetPosition {
        skill_slot: SkillSlot,
        position: Vec2,
    },
    NpcStoreTransaction {
        npc_entity_id: ClientEntityId,
        buy_items: Vec<NpcStoreBuyItem>,
        sell_items: Vec<(ItemSlot, usize)>,
    },
    RunToggle,
    SitToggle,
    DriveToggle,
    UseEmote {
        motion_id: MotionId,
        is_stop: bool,
    },
    WarpGateRequest {
        warp_gate_id: WarpGateId,
    },
    PartyCreate {
        invited_entity_id: ClientEntityId,
    },
    PartyInvite {
        invited_entity_id: ClientEntityId,
    },
    PartyLeave,
    PartyChangeOwner {
        new_owner_entity_id: ClientEntityId,
    },
    PartyKick {
        character_id: CharacterUniqueId,
    },
    PartyAcceptCreateInvite {
        owner_entity_id: ClientEntityId,
    },
    PartyAcceptJoinInvite {
        owner_entity_id: ClientEntityId,
    },
    PartyRejectInvite {
        reason: PartyRejectInviteReason,
        owner_entity_id: ClientEntityId,
    },
    PartyUpdateRules {
        item_sharing: PartyItemSharing,
        xp_sharing: PartyXpSharing,
    },
    CraftInsertGem {
        equipment_index: EquipmentIndex,
        item_slot: ItemSlot,
    },
    CraftSkillDisassemble {
        skill_slot: SkillSlot,
        item_slot: ItemSlot,
    },
    CraftNpcDisassemble {
        npc_entity_id: ClientEntityId,
        item_slot: ItemSlot,
    },
    CraftSkillUpgradeItem {
        skill_slot: SkillSlot,
        item_slot: ItemSlot,
        ingredients: [ItemSlot; 3],
    },
    CraftNpcUpgradeItem {
        npc_entity_id: ClientEntityId,
        item_slot: ItemSlot,
        ingredients: [ItemSlot; 3],
    },
    BankOpen,
    BankDepositItem {
        item_slot: ItemSlot,
        item: Item,
        is_premium: bool,
    },
    BankWithdrawItem {
        bank_slot: usize,
        item: Item,
        is_premium: bool,
    },
    RepairItemUsingItem {
        use_item_slot: ItemSlot,
        item_slot: ItemSlot,
    },
    RepairItemUsingNpc {
        npc_entity_id: ClientEntityId,
        item_slot: ItemSlot,
    },
    ClanCreate {
        name: String,
        description: String,
        mark: ClanMark,
    },
    ClanGetMemberList,
    ClanUpdateCharacterInfo {
        level: Level,
        job: u16,
    },
}
