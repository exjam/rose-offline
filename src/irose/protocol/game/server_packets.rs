use std::time::Duration;

use modular_bitfield::prelude::*;
use num_derive::FromPrimitive;

use super::common_packets::write_hotbar_slot;
use crate::{
    data::{
        item::{EquipmentItem, Item, StackableItem},
        Damage, QuestTriggerHash,
    },
    game::{
        components::{
            AmmoIndex, BasicStatType, BasicStats, CharacterInfo, ClientEntityId, Command,
            CommandData, Destination, DroppedItem, Equipment, EquipmentIndex, ExperiencePoints,
            HealthPoints, Hotbar, HotbarSlot, Inventory, InventoryPageType, ItemSlot, Level,
            ManaPoints, Money, Npc, NpcStandingDirection, Position, QuestState, SkillList,
            SkillPoints, StatPoints, Team, VehiclePartIndex, INVENTORY_PAGE_SIZE,
        },
        messages::server::{PickupDroppedItemContent, PickupDroppedItemError},
    },
    protocol::{Packet, PacketWriter},
};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    LogoutResult = 0x707,
    ConnectReply = 0x70c,
    SelectCharacter = 0x715,
    CharacterInventory = 0x716,
    UpdateInventory = 0x718,
    QuestData = 0x71b,
    JoinZone = 0x753,
    LocalChat = 0x783,
    Whisper = 0x784,
    SpawnEntityNpc = 0x791,
    SpawnEntityMonster = 0x792,
    SpawnEntityCharacter = 0x793,
    RemoveEntities = 0x794,
    StopMoveEntity = 0x796,
    AttackEntity = 0x798,
    DamageEntity = 0x799,
    MoveEntity = 0x79a,
    SpawnEntityDroppedItem = 0x7a6,
    PickupDroppedItemResult = 0x7a7,
    UpdateBasicStat = 0x7a9,
    UpdateXpStamina = 0x79b,
    UpdateLevel = 0x79e,
    UpdateEquipment = 0x7a5,
    Teleport = 0x7a8,
    SetHotbarSlot = 0x7aa,
}

trait PacketWriteEntityId {
    fn write_entity_id(&mut self, entity_id: ClientEntityId);
    fn write_option_entity_id(&mut self, entity_id: Option<ClientEntityId>);
}

impl PacketWriteEntityId for PacketWriter {
    fn write_entity_id(&mut self, entity_id: ClientEntityId) {
        self.write_u16(entity_id.0 as u16);
    }

    fn write_option_entity_id(&mut self, entity_id: Option<ClientEntityId>) {
        self.write_u16(entity_id.map_or(0, |x| x.0) as u16);
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentAmmoPart {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    #[skip]
    __: B1,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentItemPart {
    #[skip(getters)]
    item_number: B10,
    #[skip(getters)]
    gem: B9,
    #[skip(getters)]
    has_socket: bool,
    #[skip(getters)]
    grade: B4,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentItemFull {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    #[skip(getters)]
    is_crafted: bool,
    #[skip(getters)]
    gem: B9,
    #[skip(getters)]
    durability: B7,
    #[skip(getters)]
    life: B10,
    #[skip(getters)]
    has_socket: bool,
    #[skip(getters)]
    is_appraised: bool,
    #[skip(getters)]
    grade: B4,
}

#[bitfield]
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct PacketStackableItemFull {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    #[skip]
    __: B1,
    #[skip(getters)]
    quantity: B32,
}

trait PacketWriteItems {
    fn write_equipment_ammo_part(&mut self, item: Option<&StackableItem>);
    fn write_equipment_item_part(&mut self, item: Option<&EquipmentItem>);
    fn write_equipment_item_full(&mut self, equipment: Option<&EquipmentItem>);
    fn write_equipment_visible_part(&mut self, equipment: &Equipment);
    fn write_stackable_item_full(&mut self, stackable: Option<&StackableItem>);
    fn write_item_full(&mut self, item: Option<&Item>);
    fn write_item_full_money(&mut self, money: Money);
}

impl PacketWriteItems for PacketWriter {
    fn write_equipment_ammo_part(&mut self, item: Option<&StackableItem>) {
        if let Some(item) = item {
            let part = PacketEquipmentAmmoPart::new()
                .with_item_number(item.item.item_number as u16)
                .with_item_type(item.item.item_type as u8);
            for b in part.into_bytes().iter() {
                self.write_u8(*b);
            }
        } else {
            self.write_u16(0);
        }
    }

    fn write_equipment_item_part(&mut self, item: Option<&EquipmentItem>) {
        if let Some(item) = item {
            let part = PacketEquipmentItemPart::new()
                .with_item_number(item.item.item_number as u16)
                .with_gem(item.gem)
                .with_has_socket(item.has_socket)
                .with_grade(item.grade);
            for b in part.into_bytes().iter() {
                self.write_u8(*b);
            }
            self.write_u8(0);
        } else {
            self.write_u32(0);
        }
    }

    fn write_equipment_visible_part(&mut self, equipment: &Equipment) {
        for index in &[
            EquipmentIndex::Head,
            EquipmentIndex::Body,
            EquipmentIndex::Hands,
            EquipmentIndex::Feet,
            EquipmentIndex::Face,
            EquipmentIndex::Back,
            EquipmentIndex::WeaponRight,
            EquipmentIndex::WeaponLeft,
        ] {
            self.write_equipment_item_part(equipment.get_equipment_item(*index));
        }
    }

    fn write_equipment_item_full(&mut self, equipment: Option<&EquipmentItem>) {
        match equipment {
            Some(equipment) => {
                let item = PacketEquipmentItemFull::new()
                    .with_item_type(equipment.item.item_type as u8)
                    .with_item_number(equipment.item.item_number as u16)
                    .with_is_crafted(equipment.is_crafted)
                    .with_gem(equipment.gem)
                    .with_durability(equipment.durability)
                    .with_life(equipment.life)
                    .with_has_socket(equipment.has_socket)
                    .with_is_appraised(equipment.is_appraised)
                    .with_grade(equipment.grade);
                self.write_bytes(&item.into_bytes());
            }
            _ => {
                self.write_u16(0);
                self.write_u32(0);
            }
        }
    }

    fn write_stackable_item_full(&mut self, stackable: Option<&StackableItem>) {
        match stackable {
            Some(stackable) => {
                let item = PacketStackableItemFull::new()
                    .with_item_type(stackable.item.item_type as u8)
                    .with_item_number(stackable.item.item_number as u16)
                    .with_quantity(stackable.quantity);
                self.write_bytes(&item.into_bytes());
            }
            _ => {
                self.write_u16(0);
                self.write_u32(0);
            }
        }
    }

    fn write_item_full(&mut self, item: Option<&Item>) {
        match item {
            Some(Item::Equipment(equipment)) => {
                self.write_equipment_item_full(Some(equipment));
            }
            Some(Item::Stackable(stackable)) => {
                self.write_stackable_item_full(Some(stackable));
            }
            _ => {
                self.write_u16(0);
                self.write_u32(0);
            }
        }
    }

    fn write_item_full_money(&mut self, money: Money) {
        let item = PacketStackableItemFull::new()
            .with_item_type(31)
            .with_quantity(money.0 as u32);
        self.write_bytes(&item.into_bytes());
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ConnectResult {
    Ok = 0,
    Failed = 1,
    TimeOut = 2,
    InvalidPassword = 3,
    AlreadyLoggedIn = 4,
}

pub struct PacketConnectionReply {
    pub result: ConnectResult,
    pub packet_sequence_id: u32,
    pub pay_flags: u32,
}

impl From<&PacketConnectionReply> for Packet {
    fn from(packet: &PacketConnectionReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ConnectReply as u16);
        writer.write_u8(packet.result as u8);
        writer.write_u32(packet.packet_sequence_id);
        writer.write_u32(packet.pay_flags);
        writer.into()
    }
}

pub struct PacketServerSelectCharacter<'a> {
    pub character_info: &'a CharacterInfo,
    pub position: &'a Position,
    pub equipment: &'a Equipment,
    pub basic_stats: &'a BasicStats,
    pub level: &'a Level,
    pub experience_points: &'a ExperiencePoints,
    pub skill_list: &'a SkillList,
    pub hotbar: &'a Hotbar,
    pub health_points: &'a HealthPoints,
    pub mana_points: &'a ManaPoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
}

impl<'a> From<&'a PacketServerSelectCharacter<'a>> for Packet {
    fn from(packet: &'a PacketServerSelectCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SelectCharacter as u16);
        let character_info = packet.character_info;
        writer.write_u8(character_info.gender);
        writer.write_u16(packet.position.zone);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_u16(character_info.revive_zone);

        writer.write_u32(character_info.face as u32);
        writer.write_u32(character_info.hair as u32);
        writer.write_equipment_visible_part(&packet.equipment);

        // tagBasicInfo
        writer.write_u8(character_info.birth_stone);
        writer.write_u8(character_info.face as u8);
        writer.write_u8(character_info.hair as u8);
        writer.write_u16(character_info.job);
        writer.write_u8(0); // union
        writer.write_u8(0); // rank
        writer.write_u8(0); // fame

        // tagBasicAbility
        let basic_stats = packet.basic_stats;
        writer.write_u16(basic_stats.strength);
        writer.write_u16(basic_stats.dexterity);
        writer.write_u16(basic_stats.intelligence);
        writer.write_u16(basic_stats.concentration);
        writer.write_u16(basic_stats.charm);
        writer.write_u16(basic_stats.sense);

        // tagGrowAbility
        writer.write_u16(packet.health_points.hp as u16);
        writer.write_u16(packet.mana_points.mp as u16);
        writer.write_u32(packet.experience_points.xp as u32);
        writer.write_u16(packet.level.level as u16);
        writer.write_u16(packet.stat_points.points as u16);
        writer.write_u16(packet.skill_points.points as u16);
        writer.write_u8(100); // Body Size
        writer.write_u8(200); // Head Size
        writer.write_u32(0); // Penalty XP
        writer.write_u16(0); // Fame G
        writer.write_u16(0); // Fame B
        for _ in 0..10 {
            writer.write_u16(0); // Union points
        }
        writer.write_u32(0); // Guild ID
        writer.write_u16(0); // Guild contribution
        writer.write_u8(0); // Guild pos
        writer.write_u16(0); // PK flag
        writer.write_u16(100); // Stamina

        for _ in 0..40 {
            writer.write_u32(0); // seconds remaining
            writer.write_u16(0); // buff id
            writer.write_u16(0); // reserved
        }

        // tagSkillAbility
        assert!(packet.skill_list.pages.len() * packet.skill_list.pages[0].len() == 120);
        for page in &packet.skill_list.pages {
            for slot in page {
                writer.write_u16(slot.map_or(0, |x| x.0) as u16);
            }
        }

        // CHotIcons
        assert!(packet.hotbar.pages.len() * packet.hotbar.pages[0].len() == 32);
        for page in &packet.hotbar.pages {
            for slot in page {
                write_hotbar_slot(&mut writer, slot);
            }
        }

        writer.write_u32(123); // server wide unique id
        writer.write_null_terminated_utf8(&character_info.name);
        writer.into()
    }
}

pub struct PacketServerCharacterInventory<'a> {
    pub equipment: &'a Equipment,
    pub inventory: &'a Inventory,
}

impl<'a> From<&'a PacketServerCharacterInventory<'a>> for Packet {
    fn from(packet: &'a PacketServerCharacterInventory<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CharacterInventory as u16);
        let inventory = packet.inventory;
        let equipment = packet.equipment;
        writer.write_i64(inventory.money.0);

        for item in &equipment.equipped_items {
            writer.write_equipment_item_full(item.as_ref());
        }

        for item in &inventory.equipment.slots {
            writer.write_item_full(item.as_ref());
        }

        for item in &inventory.consumables.slots {
            writer.write_item_full(item.as_ref());
        }

        for item in &inventory.materials.slots {
            writer.write_item_full(item.as_ref());
        }

        for item in &inventory.vehicles.slots {
            writer.write_item_full(item.as_ref());
        }

        for item in &equipment.equipped_ammo {
            writer.write_stackable_item_full(item.as_ref());
        }

        for item in &equipment.equipped_vehicle {
            writer.write_equipment_item_full(item.as_ref());
        }

        writer.into()
    }
}

pub struct PacketServerCharacterQuestData<'a> {
    pub quest_state: &'a QuestState,
}

impl<'a> From<&'a PacketServerCharacterQuestData<'a>> for Packet {
    fn from(packet: &'a PacketServerCharacterQuestData<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::QuestData as u16);

        for i in 0..5 {
            writer.write_u16(
                packet
                    .quest_state
                    .episode_variables
                    .get(i)
                    .cloned()
                    .unwrap_or(0u16),
            );
        }

        for i in 0..3 {
            writer.write_u16(
                packet
                    .quest_state
                    .job_variables
                    .get(i)
                    .cloned()
                    .unwrap_or(0u16),
            );
        }

        for i in 0..7 {
            writer.write_u16(
                packet
                    .quest_state
                    .planet_variables
                    .get(i)
                    .cloned()
                    .unwrap_or(0u16),
            );
        }

        for i in 0..10 {
            writer.write_u16(
                packet
                    .quest_state
                    .union_variables
                    .get(i)
                    .cloned()
                    .unwrap_or(0u16),
            );
        }

        for i in 0..10 {
            let quest = packet
                .quest_state
                .active_quests
                .get(i)
                .and_then(|q| q.as_ref());
            // Quest data
            writer.write_u16(quest.map_or(0, |quest| quest.quest_id));
            writer.write_u32(quest.and_then(|quest| quest.expire_time).unwrap_or(0));
            for j in 0..10 {
                writer.write_u16(
                    quest
                        .and_then(|quest| quest.variables.get(j).cloned())
                        .unwrap_or(0),
                );
            }
            writer.write_u32(quest.map_or(0, |quest| quest.switches.as_buffer()[0]));
            for j in 0..6 {
                writer.write_item_full(
                    quest.and_then(|quest| quest.items.get(j).and_then(|item| item.as_ref())),
                );
            }
        }

        for _ in 0..30 {
            writer.write_item_full(None); // wish list items
        }

        writer.into()
    }
}

pub struct PacketServerAttackEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: ClientEntityId,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl From<&PacketServerAttackEntity> for Packet {
    fn from(packet: &PacketServerAttackEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::AttackEntity as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.target_entity_id);
        writer.write_u16(packet.distance);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);
        writer.into()
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketServerDamage {
    #[skip(getters)]
    amount: B11,
    #[skip(getters)]
    action: B5,
}

pub struct PacketServerDamageEntity {
    pub attacker_entity_id: ClientEntityId,
    pub defender_entity_id: ClientEntityId,
    pub damage: Damage,
    pub is_killed: bool,
    // TODO: Optional item drop with damage
}

impl From<&PacketServerDamageEntity> for Packet {
    fn from(packet: &PacketServerDamageEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::DamageEntity as u16);
        writer.write_entity_id(packet.attacker_entity_id);
        writer.write_entity_id(packet.defender_entity_id);

        let mut action = 0u8;
        if packet.damage.is_critical {
            action |= 0x08;
        }
        if packet.damage.apply_hit_stun {
            action |= 0x04;
        }
        if packet.is_killed {
            action |= 0x10;
        }

        let damage = PacketServerDamage::new()
            .with_amount(packet.damage.amount as u16)
            .with_action(action);

        for b in damage.into_bytes().iter() {
            writer.write_u8(*b);
        }
        writer.into()
    }
}

pub struct PacketServerMoveEntity {
    pub entity_id: ClientEntityId,
    pub target_entity_id: Option<ClientEntityId>,
    pub distance: u16,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl From<&PacketServerMoveEntity> for Packet {
    fn from(packet: &PacketServerMoveEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::MoveEntity as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_u16(packet.distance);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);
        writer.into()
    }
}

pub struct PacketServerJoinZone<'a> {
    pub entity_id: ClientEntityId,
    pub level: &'a Level,
    pub experience_points: &'a ExperiencePoints,
    pub team: &'a Team,
    pub health_points: &'a HealthPoints,
    pub mana_points: &'a ManaPoints,
}

impl<'a> From<&'a PacketServerJoinZone<'a>> for Packet {
    fn from(packet: &'a PacketServerJoinZone<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::JoinZone as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.health_points.hp as u16);
        writer.write_u16(packet.mana_points.mp as u16);

        writer.write_u32(packet.experience_points.xp as u32);
        writer.write_u32(0); // penalty xp

        // tagVAR_GLOBAL
        writer.write_u16(100); // craft rate
        writer.write_u32(0); // update time
        writer.write_u16(100); // world price rate
        writer.write_u8(100); // town rate
        for _ in 0..11 {
            writer.write_u8(100); // item rate
        }
        writer.write_u32(0); // global flags (0x1 = pvp allowed)

        writer.write_u32(0); // account world time
        writer.write_u32(packet.team.id);
        writer.into()
    }
}

pub struct PacketServerLocalChat<'a> {
    pub entity_id: ClientEntityId,
    pub text: &'a str,
}

impl<'a> From<&'a PacketServerLocalChat<'a>> for Packet {
    fn from(packet: &'a PacketServerLocalChat<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LocalChat as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_null_terminated_utf8(packet.text);
        writer.into()
    }
}

pub struct PacketServerWhisper<'a> {
    pub from: &'a str,
    pub text: &'a str,
}

impl<'a> From<&'a PacketServerWhisper<'a>> for Packet {
    fn from(packet: &'a PacketServerWhisper<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::Whisper as u16);
        writer.write_null_terminated_utf8(packet.from);
        writer.write_null_terminated_utf8(packet.text);
        writer.into()
    }
}

pub struct PacketServerStopMoveEntity {
    pub entity_id: ClientEntityId,
    pub x: f32,
    pub y: f32,
    pub z: u16,
}

impl From<&PacketServerStopMoveEntity> for Packet {
    fn from(packet: &PacketServerStopMoveEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::StopMoveEntity as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);
        writer.into()
    }
}

pub struct PacketServerTeleport {
    pub entity_id: ClientEntityId,
    pub zone_no: u16,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

impl From<&PacketServerTeleport> for Packet {
    fn from(packet: &PacketServerTeleport) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::Teleport as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.zone_no);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u8(packet.run_mode);
        writer.write_u8(packet.ride_mode);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketServerSetHotbarSlot {
    pub slot_index: u8,
    pub slot: Option<HotbarSlot>,
}

impl From<&PacketServerSetHotbarSlot> for Packet {
    fn from(packet: &PacketServerSetHotbarSlot) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SetHotbarSlot as u16);
        writer.write_u8(packet.slot_index);
        write_hotbar_slot(&mut writer, &packet.slot);
        writer.into()
    }
}

trait PacketWriteCommand {
    fn write_command_id(&mut self, command: &Command);
}

impl PacketWriteCommand for PacketWriter {
    fn write_command_id(&mut self, command: &Command) {
        let command_id = match command.command {
            CommandData::Stop => 0,
            CommandData::Move(_) => 1,
            CommandData::Attack(_) => 2,
            CommandData::Die(_) => 3,
            CommandData::PickupDroppedItem(_) => 4,
            // 6 = Cast skill on self
            // 7 = Skill on entity
            // 8 = Skill on position
            // 9 = Run away
            // 10 = Sit
            // 11 = Vending Shop
        };
        self.write_u16(command_id);
    }
}

pub struct PacketServerSpawnEntityDroppedItem<'a> {
    pub entity_id: ClientEntityId,
    pub dropped_item: &'a DroppedItem,
    pub position: &'a Position,
    pub owner_entity_id: Option<ClientEntityId>,
    pub remaining_time: &'a Duration,
}

impl<'a> From<&'a PacketServerSpawnEntityDroppedItem<'a>> for Packet {
    fn from(packet: &'a PacketServerSpawnEntityDroppedItem<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityDroppedItem as u16);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        match packet.dropped_item {
            DroppedItem::Item(item) => writer.write_item_full(Some(item)),
            DroppedItem::Money(amount) => writer.write_item_full_money(*amount),
        }
        writer.write_entity_id(packet.entity_id);
        writer.write_option_entity_id(packet.owner_entity_id);
        writer.write_u16(packet.remaining_time.as_millis() as u16);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityNpc<'a> {
    pub entity_id: ClientEntityId,
    pub npc: &'a Npc,
    pub direction: &'a NpcStandingDirection,
    pub position: &'a Position,
    pub team: &'a Team,
    pub destination: Option<&'a Destination>,
    pub command: &'a Command,
    pub target_entity_id: Option<ClientEntityId>,
    pub health: &'a HealthPoints,
}

impl<'a> From<&'a PacketServerSpawnEntityNpc<'a>> for Packet {
    fn from(packet: &'a PacketServerSpawnEntityNpc<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityNpc as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.x));
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.y));
        writer.write_command_id(packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_u8(0); // move mode
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_u32(0); // status flag
        writer.write_u16(packet.npc.id as u16);
        writer.write_u16(packet.npc.quest_index);
        writer.write_f32(packet.direction.direction);
        writer.write_u16(0); // event status
        writer.into()
    }
}

pub struct PacketServerSpawnEntityMonster<'a> {
    pub entity_id: ClientEntityId,
    pub npc: &'a Npc,
    pub position: &'a Position,
    pub destination: Option<&'a Destination>,
    pub team: &'a Team,
    pub health: &'a HealthPoints,
    pub command: &'a Command,
    pub target_entity_id: Option<ClientEntityId>,
}

impl<'a> From<&'a PacketServerSpawnEntityMonster<'a>> for Packet {
    fn from(packet: &'a PacketServerSpawnEntityMonster<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityMonster as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.x));
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.y));
        writer.write_command_id(packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_u8(0); // move mode
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_u32(0); // status flag
        writer.write_u16(packet.npc.id as u16);
        writer.write_u16(packet.npc.quest_index);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityCharacter<'a> {
    pub character_info: &'a CharacterInfo,
    pub command: &'a Command,
    pub destination: Option<&'a Destination>,
    pub entity_id: ClientEntityId,
    pub equipment: &'a Equipment,
    pub health: &'a HealthPoints,
    pub level: &'a Level,
    pub passive_attack_speed: i32,
    pub position: &'a Position,
    pub run_speed: f32,
    pub target_entity_id: Option<ClientEntityId>,
    pub team: &'a Team,
}

impl<'a> From<&'a PacketServerSpawnEntityCharacter<'a>> for Packet {
    fn from(packet: &'a PacketServerSpawnEntityCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityCharacter as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.x));
        writer.write_f32(packet.destination.map_or(0.0, |d| d.position.y));
        writer.write_command_id(packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_u8(0); // move mode
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_u32(0); // status flag
        writer.write_u8(packet.character_info.gender);
        writer.write_u16(packet.run_speed as u16);
        writer.write_u16(packet.passive_attack_speed as u16);
        writer.write_u8(0); // weight rate

        writer.write_u32(packet.character_info.face as u32);
        writer.write_u32(packet.character_info.hair as u32);
        writer.write_equipment_visible_part(&packet.equipment);

        for index in &[AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            writer.write_equipment_ammo_part(packet.equipment.get_ammo_item(*index));
        }

        writer.write_u16(packet.character_info.job as u16);
        writer.write_u8(packet.level.level as u8);

        for index in &[
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Ability,
            VehiclePartIndex::Arms,
        ] {
            writer.write_equipment_item_part(packet.equipment.get_vehicle_item(*index));
        }

        writer.write_u16(packet.position.position.z as u16); // z
        writer.write_u32(0); // sub flag
        writer.write_null_terminated_utf8(&packet.character_info.name);
        // if status flag != 0 then u16[] of endurace things
        // if sub flag == store then u16 type, str shop name
        // optional clan info of u32 clan id, u32 clan mark, u8 clan level, u8 clan rank
        writer.into()
    }
}

pub struct PacketServerRemoveEntities<'a> {
    pub entity_ids: &'a [ClientEntityId],
}

impl<'a> From<&'a PacketServerRemoveEntities<'a>> for Packet {
    fn from(packet: &'a PacketServerRemoveEntities<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RemoveEntities as u16);
        for entity_id in packet.entity_ids {
            writer.write_entity_id(*entity_id);
        }
        writer.into()
    }
}

fn item_slot_to_index(slot: ItemSlot) -> usize {
    match slot {
        ItemSlot::Equipped(equipment_index) => equipment_index as usize,
        ItemSlot::Inventory(page_type, index) => match page_type {
            InventoryPageType::Equipment => 12 + index,
            InventoryPageType::Consumables => 12 + INVENTORY_PAGE_SIZE + index,
            InventoryPageType::Materials => 12 + 2 * INVENTORY_PAGE_SIZE + index,
            InventoryPageType::Vehicles => 12 + 3 * INVENTORY_PAGE_SIZE + index,
        },
    }
}

pub struct PacketServerUpdateInventory<'a> {
    pub items: &'a [(ItemSlot, Option<Item>)],
}

impl<'a> From<&'a PacketServerUpdateInventory<'a>> for Packet {
    fn from(packet: &'a PacketServerUpdateInventory<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateInventory as u16);
        writer.write_u8(packet.items.len() as u8);
        for (slot, item) in packet.items {
            writer.write_u8(item_slot_to_index(*slot) as u8);
            writer.write_item_full(item.as_ref());
        }
        writer.into()
    }
}

pub struct PacketServerUpdateEquipment {
    pub entity_id: ClientEntityId,
    pub equipment_index: EquipmentIndex,
    pub item: Option<EquipmentItem>,
    pub run_speed: Option<u16>,
}

impl From<&PacketServerUpdateEquipment> for Packet {
    fn from(packet: &PacketServerUpdateEquipment) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateEquipment as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.equipment_index as u16);
        writer.write_equipment_item_part(packet.item.as_ref());
        if let Some(run_speed) = packet.run_speed {
            writer.write_u16(run_speed);
        }
        writer.into()
    }
}

pub struct PacketServerUpdateLevel {
    pub entity_id: ClientEntityId,
    pub level: Level,
    pub experience_points: ExperiencePoints,
    pub stat_points: StatPoints,
    pub skill_points: SkillPoints,
}

impl From<&PacketServerUpdateLevel> for Packet {
    fn from(packet: &PacketServerUpdateLevel) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateLevel as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.level.level as u16);
        writer.write_u32(packet.experience_points.xp as u32);
        writer.write_u16(packet.stat_points.points as u16);
        writer.write_u16(packet.skill_points.points as u16);
        writer.into()
    }
}

pub struct PacketServerUpdateXpStamina {
    pub xp: u64,
    pub stamina: u32,
    pub source_entity_id: Option<ClientEntityId>,
}

impl From<&PacketServerUpdateXpStamina> for Packet {
    fn from(packet: &PacketServerUpdateXpStamina) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateXpStamina as u16);
        writer.write_u32(packet.xp as u32);
        writer.write_u16(packet.stamina as u16);
        writer.write_option_entity_id(packet.source_entity_id);
        writer.into()
    }
}

pub struct PacketServerUpdateBasicStat {
    pub basic_stat_type: BasicStatType,
    pub value: u16,
}

impl From<&PacketServerUpdateBasicStat> for Packet {
    fn from(packet: &PacketServerUpdateBasicStat) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateBasicStat as u16);
        let id = match packet.basic_stat_type {
            BasicStatType::Strength => 0,
            BasicStatType::Dexterity => 1,
            BasicStatType::Intelligence => 2,
            BasicStatType::Concentration => 3,
            BasicStatType::Charm => 4,
            BasicStatType::Sense => 5,
        };
        writer.write_u8(id);
        writer.write_u16(packet.value);
        writer.into()
    }
}

pub struct PacketServerPickupDroppedItemResult {
    pub item_entity_id: ClientEntityId,
    pub result: Result<PickupDroppedItemContent, PickupDroppedItemError>,
}

impl From<&PacketServerPickupDroppedItemResult> for Packet {
    fn from(packet: &PacketServerPickupDroppedItemResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PickupDroppedItemResult as u16);
        writer.write_entity_id(packet.item_entity_id);
        match &packet.result {
            Ok(PickupDroppedItemContent::Item(slot, item)) => {
                writer.write_u8(0); // OK
                writer.write_u16(item_slot_to_index(*slot) as u16);
                writer.write_item_full(Some(item));
            }
            Ok(PickupDroppedItemContent::Money(money)) => {
                writer.write_u8(0); // OK
                writer.write_u16(0); // Slot
                writer.write_item_full_money(*money);
            }
            Err(error) => {
                match error {
                    PickupDroppedItemError::NotExist => writer.write_u8(1),
                    PickupDroppedItemError::NoPermission => writer.write_u8(2),
                    PickupDroppedItemError::InventoryFull => writer.write_u8(3),
                }
                writer.write_u16(0); // Slot
                writer.write_item_full(None);
            }
        };
        writer.into()
    }
}

pub struct PacketServerLogoutResult {
    pub result: Result<(), Duration>,
}

impl From<&PacketServerLogoutResult> for Packet {
    fn from(packet: &PacketServerLogoutResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LogoutResult as u16);
        match &packet.result {
            Ok(_) => {
                writer.write_u16(0);
            }
            Err(duration) => {
                writer.write_u16(duration.as_secs() as u16);
            }
        };
        writer.into()
    }
}
