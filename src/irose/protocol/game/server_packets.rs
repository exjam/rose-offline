use std::time::Duration;

use modular_bitfield::prelude::*;
use nalgebra::Point2;
use num_derive::FromPrimitive;

use crate::{
    data::{
        item::{EquipmentItem, Item},
        AbilityType, Damage, ItemReference, NpcId, SkillId, WorldTicks, ZoneId,
    },
    game::{
        components::{
            AmmoIndex, BasicStatType, BasicStats, CharacterInfo, ClientEntityId, Command,
            CommandCastSkill, CommandCastSkillTarget, CommandData, Destination, DroppedItem,
            Equipment, EquipmentIndex, ExperiencePoints, HealthPoints, Hotbar, HotbarSlot,
            Inventory, ItemSlot, Level, ManaPoints, Money, MoveMode, MoveSpeed, Npc,
            NpcStandingDirection, Position, QuestState, SkillList, SkillPage, SkillPoints, Stamina,
            StatPoints, StatusEffects, Team, UnionMembership, VehiclePartIndex,
        },
        messages::server::{
            CancelCastingSkillReason, LearnSkillError, LearnSkillSuccess, PickupDroppedItemContent,
            PickupDroppedItemError,
        },
    },
    irose::protocol::game::common_packets::{
        PacketWriteDamage, PacketWriteHotbarSlot, PacketWriteItemSlot, PacketWriteItems,
        PacketWriteMoveMode, PacketWriteSkillSlot, PacketWriteStatusEffects,
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
    UpdateMoney = 0x71d,
    UpdateMoneyReward = 0x71e,
    UpdateInventoryReward = 0x71f,
    UpdateAbilityValueRewardAdd = 0x720,
    UpdateAbilityValueRewardSet = 0x721,
    QuestResult = 0x730,
    RunNpcDeathTrigger = 0x731,
    JoinZone = 0x753,
    LocalChat = 0x783,
    Whisper = 0x784,
    SpawnEntityNpc = 0x791,
    SpawnEntityMonster = 0x792,
    SpawnEntityCharacter = 0x793,
    RemoveEntities = 0x794,
    StopMoveEntity = 0x796,
    MoveEntityWithMoveMode = 0x797,
    AttackEntity = 0x798,
    DamageEntity = 0x799,
    MoveEntity = 0x79a,
    UpdateXpStamina = 0x79b,
    UpdateLevel = 0x79e,
    UseItem = 0x7a3,
    UpdateEquipment = 0x7a5,
    SpawnEntityDroppedItem = 0x7a6,
    PickupDroppedItemResult = 0x7a7,
    Teleport = 0x7a8,
    UpdateBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
    LearnSkillResult = 0x7b0,
    CastSkillSelf = 0x7b2,
    CastSkillTargetEntity = 0x7b3,
    CastSkillTargetPosition = 0x7b4,
    ApplySkillEffect = 0x7b5,
    ApplySkillDamage = 0x7b6,
    UpdateStatusEffects = 0x7b7,
    UpdateSpeed = 0x7b8,
    FinishCastingSkill = 0x7b9,
    StartCastingSkill = 0x7bb,
    CancelCastingSkill = 0x7bd,
    OpenPersonalStore = 0x7c2,
    PersonalStoreItemList = 0x7c4,
    PersonalStoreTransactionResult = 0x7c6,
    PersonalStoreTransactionUpdateMoneyAndInventory = 0x7c7,
    MoveToggle = 0x782,
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

fn write_skill_page(writer: &mut PacketWriter, skill_page: &SkillPage) {
    for index in 0..30 {
        writer.write_u16(
            skill_page
                .skills
                .get(index)
                .copied()
                .flatten()
                .map_or(0, |x| x.get()) as u16,
        );
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
    pub union_membership: &'a UnionMembership,
    pub stamina: Stamina,
}

impl<'a> From<&'a PacketServerSelectCharacter<'a>> for Packet {
    fn from(packet: &'a PacketServerSelectCharacter<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SelectCharacter as u16);
        let character_info = packet.character_info;
        writer.write_u8(character_info.gender);
        writer.write_u16(packet.position.zone_id.get() as u16);
        writer.write_f32(packet.position.position.x);
        writer.write_f32(packet.position.position.y);
        writer.write_u16(character_info.revive_zone_id.get() as u16);

        writer.write_u32(character_info.face as u32);
        writer.write_u32(character_info.hair as u32);
        writer.write_equipment_visible_part(packet.equipment);

        // tagBasicInfo
        writer.write_u8(character_info.birth_stone);
        writer.write_u8(character_info.face as u8);
        writer.write_u8(character_info.hair as u8);
        writer.write_u16(character_info.job);
        writer.write_u8(packet.union_membership.current_union.unwrap_or(0) as u8);
        writer.write_u8(character_info.rank);
        writer.write_u8(character_info.fame);

        // tagBasicAbility
        let basic_stats = packet.basic_stats;
        writer.write_u16(basic_stats.strength as u16);
        writer.write_u16(basic_stats.dexterity as u16);
        writer.write_u16(basic_stats.intelligence as u16);
        writer.write_u16(basic_stats.concentration as u16);
        writer.write_u16(basic_stats.charm as u16);
        writer.write_u16(basic_stats.sense as u16);

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
        writer.write_u16(character_info.fame_g);
        writer.write_u16(character_info.fame_b);

        for i in 0..10 {
            writer.write_u16(
                packet
                    .union_membership
                    .points
                    .get(i)
                    .cloned()
                    .unwrap_or(0u32) as u16,
            );
        }

        writer.write_u32(0); // Guild ID
        writer.write_u16(0); // Guild contribution
        writer.write_u8(0); // Guild pos
        writer.write_u16(0); // PK flag
        writer.write_u16(packet.stamina.stamina as u16);

        for _ in 0..40 {
            writer.write_u32(0); // seconds remaining
            writer.write_u16(0); // buff id
            writer.write_u16(0); // reserved
        }

        // tagSkillAbility
        write_skill_page(&mut writer, &packet.skill_list.basic);
        write_skill_page(&mut writer, &packet.skill_list.active);
        write_skill_page(&mut writer, &packet.skill_list.passive);
        write_skill_page(&mut writer, &packet.skill_list.clan);

        // CHotIcons
        assert!(packet.hotbar.pages.len() * packet.hotbar.pages[0].len() == 32);
        for page in &packet.hotbar.pages {
            for slot in page {
                writer.write_hotbar_slot(slot);
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

        // Episode Variables
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

        // Job Variables
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

        // Planet Variables
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

        // Union Variables
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

        // Active Quests
        for i in 0..10 {
            let quest = packet
                .quest_state
                .active_quests
                .get(i)
                .and_then(|q| q.as_ref());

            // Active Quest Data
            writer.write_u16(quest.map_or(0, |quest| quest.quest_id) as u16);
            writer.write_u32(
                quest
                    .and_then(|quest| quest.expire_time)
                    .map(|expire_time| expire_time.0 as u32)
                    .unwrap_or(0),
            );

            // Active Quest Variables
            for j in 0..10 {
                writer.write_u16(
                    quest
                        .and_then(|quest| quest.variables.get(j).cloned())
                        .unwrap_or(0),
                );
            }

            // Active Quest Switches
            writer.write_u32(quest.map_or(0, |quest| quest.switches.as_buffer()[0]));

            // Active Quest Items
            for j in 0..6 {
                writer.write_item_full(
                    quest.and_then(|quest| quest.items.get(j).and_then(|item| item.as_ref())),
                );
            }
        }

        // Quest Switches
        let quest_switches_u32 = packet.quest_state.quest_switches.as_buffer();
        for i in 0..(1024 / 32) {
            writer.write_u32(quest_switches_u32.get(i).cloned().unwrap_or(0));
        }

        for _ in 0..30 {
            writer.write_item_full(None); // TODO: Wish list items
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
        writer.write_damage_u16(&packet.damage, packet.is_killed);
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
    pub move_mode: Option<MoveMode>,
}

impl From<&PacketServerMoveEntity> for Packet {
    fn from(packet: &PacketServerMoveEntity) -> Self {
        let opcode = if packet.move_mode.is_some() {
            ServerPackets::MoveEntityWithMoveMode
        } else {
            ServerPackets::MoveEntity
        };
        let mut writer = PacketWriter::new(opcode as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_u16(packet.distance);
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u16(packet.z);

        if let Some(move_mode) = packet.move_mode {
            writer.write_move_mode_u8(move_mode);
        }

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
    pub world_ticks: WorldTicks,
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

        writer.write_u32(packet.world_ticks.0 as u32);
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
    pub zone_id: ZoneId,
    pub x: f32,
    pub y: f32,
    pub run_mode: u8,
    pub ride_mode: u8,
}

impl From<&PacketServerTeleport> for Packet {
    fn from(packet: &PacketServerTeleport) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::Teleport as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.zone_id.get() as u16);
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
        writer.write_hotbar_slot(&packet.slot);
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
            CommandData::PersonalStore => 11,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: None, ..
            }) => 6,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: Some(CommandCastSkillTarget::Entity(_)),
                ..
            }) => 7,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: Some(CommandCastSkillTarget::Position(_)),
                ..
            }) => 8,
            // 9 = Run away
            // 10 = Sit
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
    pub move_mode: MoveMode,
    pub status_effects: &'a StatusEffects,
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
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(packet.status_effects);
        writer.write_u16(packet.npc.id.get() as u16);
        writer.write_u16(packet.npc.quest_index);
        writer.write_f32(packet.direction.direction);
        writer.write_u16(0); // event status
        writer.write_status_effects_values(packet.status_effects);
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
    pub move_mode: MoveMode,
    pub status_effects: &'a StatusEffects,
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
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(packet.status_effects);
        writer.write_u16(packet.npc.id.get() as u16);
        writer.write_u16(packet.npc.quest_index);
        writer.write_status_effects_values(packet.status_effects);
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
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub passive_attack_speed: i32,
    pub position: &'a Position,
    pub status_effects: &'a StatusEffects,
    pub target_entity_id: Option<ClientEntityId>,
    pub team: &'a Team,
    pub personal_store_info: &'a Option<(i32, String)>,
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
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_u32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(packet.status_effects);
        writer.write_u8(packet.character_info.gender);
        writer.write_u16(packet.move_speed.speed as u16);
        writer.write_u16(packet.passive_attack_speed as u16);
        writer.write_u8(0); // TODO: Weight rate

        writer.write_u32(packet.character_info.face as u32);
        writer.write_u32(packet.character_info.hair as u32);
        writer.write_equipment_visible_part(packet.equipment);

        for index in &[AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            writer.write_equipment_ammo_part(packet.equipment.get_ammo_item(*index));
        }

        writer.write_u16(packet.character_info.job as u16);
        writer.write_u8(packet.level.level as u8);

        for index in &[
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Arms,
        ] {
            writer.write_equipment_item_part(packet.equipment.get_vehicle_item(*index));
        }

        writer.write_u16(packet.position.position.z as u16);

        /*
        TODO Sub flags:
        Hide = 1,
        PersonalStore = 2,
        IntroChat = 4,
        AruaFairy = 0x40000000,
        */
        let mut sub_flags = 0;
        if packet.personal_store_info.is_some() {
            sub_flags |= 0x2;
        }
        writer.write_u32(sub_flags);
        writer.write_null_terminated_utf8(&packet.character_info.name);

        writer.write_status_effects_values(packet.status_effects);

        if let Some((personal_store_skin, personal_store_title)) = packet.personal_store_info {
            writer.write_u16(*personal_store_skin as u16);
            writer.write_null_terminated_utf8(personal_store_title);
        }

        // TODO: Clan info - u32 clan id, u32 clan mark, u8 clan level, u8 clan rank
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

pub struct PacketServerUpdateInventory<'a> {
    pub is_reward: bool,
    pub items: &'a [(ItemSlot, Option<Item>)],
}

impl<'a> From<&'a PacketServerUpdateInventory<'a>> for Packet {
    fn from(packet: &'a PacketServerUpdateInventory<'a>) -> Self {
        let command = if packet.is_reward {
            ServerPackets::UpdateInventoryReward
        } else {
            ServerPackets::UpdateInventory
        };
        let mut writer = PacketWriter::new(command as u16);
        writer.write_u8(packet.items.len() as u8);
        for (slot, item) in packet.items {
            writer.write_item_slot_u8(*slot);
            writer.write_item_full(item.as_ref());
        }
        writer.into()
    }
}

pub struct PacketServerUpdateMoney {
    pub is_reward: bool,
    pub money: Money,
}

impl From<&PacketServerUpdateMoney> for Packet {
    fn from(packet: &PacketServerUpdateMoney) -> Self {
        let command = if packet.is_reward {
            ServerPackets::UpdateMoneyReward
        } else {
            ServerPackets::UpdateMoney
        };
        let mut writer = PacketWriter::new(command as u16);
        writer.write_i64(packet.money.0);
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
    pub value: i32,
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
        writer.write_u16(packet.value as u16);
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
                writer.write_item_slot_u16(*slot);
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

#[allow(dead_code)]
pub enum PacketServerQuestResultType {
    AddSuccess,
    AddFailed,
    DeleteSuccess,
    DeleteFailed,
    TriggerSuccess,
    TriggerFailed,
}

pub struct PacketServerQuestResult {
    pub result: PacketServerQuestResultType,
    pub slot: u8,
    pub quest_id: u32,
}

impl From<&PacketServerQuestResult> for Packet {
    fn from(packet: &PacketServerQuestResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::QuestResult as u16);
        writer.write_u8(match packet.result {
            PacketServerQuestResultType::AddSuccess => 1,
            PacketServerQuestResultType::AddFailed => 2,
            PacketServerQuestResultType::DeleteSuccess => 3,
            PacketServerQuestResultType::DeleteFailed => 4,
            PacketServerQuestResultType::TriggerSuccess => 5,
            PacketServerQuestResultType::TriggerFailed => 6,
        } as u8);
        writer.write_u8(packet.slot);
        writer.write_u32(packet.quest_id);
        writer.into()
    }
}

pub struct PacketServerUpdateAbilityValue {
    pub is_add: bool,
    pub ability_type: AbilityType,
    pub value: i32,
}

impl From<&PacketServerUpdateAbilityValue> for Packet {
    fn from(packet: &PacketServerUpdateAbilityValue) -> Self {
        let command = if packet.is_add {
            ServerPackets::UpdateAbilityValueRewardAdd
        } else {
            ServerPackets::UpdateAbilityValueRewardSet
        };
        let mut writer = PacketWriter::new(command as u16);
        writer.write_u16(packet.ability_type as u16);
        writer.write_i32(packet.value);
        writer.into()
    }
}

pub struct PacketServerLearnSkillResult {
    pub result: Result<LearnSkillSuccess, LearnSkillError>,
}

impl From<&PacketServerLearnSkillResult> for Packet {
    fn from(packet: &PacketServerLearnSkillResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LearnSkillResult as u16);
        match packet.result {
            Ok(LearnSkillSuccess {
                skill_slot,
                skill_id,
                updated_skill_points,
            }) => {
                writer.write_u8(1); // Success
                writer.write_skill_slot_u8(skill_slot);
                writer.write_u16(skill_id.get() as u16);
                writer.write_u16(updated_skill_points.points as u16);
            }
            Err(error) => {
                match error {
                    LearnSkillError::AlreadyLearnt => writer.write_u8(0),
                    LearnSkillError::JobRequirement => writer.write_u8(2),
                    LearnSkillError::SkillRequirement => writer.write_u8(3),
                    LearnSkillError::AbilityRequirement => writer.write_u8(4),
                    LearnSkillError::Full => writer.write_u8(5),
                    LearnSkillError::InvalidSkillId => writer.write_u8(6),
                    LearnSkillError::SkillPointRequirement => writer.write_u8(7),
                }
                writer.write_u8(0);
                writer.write_u16(0);
                writer.write_u16(0);
            }
        }
        writer.into()
    }
}

pub struct PacketServerRunNpcDeathTrigger {
    pub npc_id: NpcId,
}

impl From<&PacketServerRunNpcDeathTrigger> for Packet {
    fn from(packet: &PacketServerRunNpcDeathTrigger) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RunNpcDeathTrigger as u16);
        writer.write_u16(packet.npc_id.get() as u16);
        writer.into()
    }
}

pub struct PacketServerOpenPersonalStore<'a> {
    pub entity_id: ClientEntityId,
    pub skin: i32,
    pub title: &'a str,
}

impl<'a> From<&'a PacketServerOpenPersonalStore<'a>> for Packet {
    fn from(packet: &'a PacketServerOpenPersonalStore<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::OpenPersonalStore as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skin as u16);
        writer.write_null_terminated_utf8(packet.title);
        writer.into()
    }
}

pub struct PacketServerPersonalStoreItemList<'a> {
    pub sell_items: &'a [(u8, Item, Money)],
    pub buy_items: &'a [(u8, Item, Money)],
}

impl<'a> From<&'a PacketServerPersonalStoreItemList<'a>> for Packet {
    fn from(packet: &'a PacketServerPersonalStoreItemList<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PersonalStoreItemList as u16);

        writer.write_u8(packet.sell_items.len() as u8);
        writer.write_u8(packet.buy_items.len() as u8);

        for (slot_index, item, price) in packet.sell_items.iter() {
            writer.write_u8(*slot_index);
            writer.write_item_full(Some(item));
            writer.write_u32(price.0 as u32);
        }

        for (slot_index, item, price) in packet.buy_items.iter() {
            writer.write_u8(*slot_index);
            writer.write_item_full(Some(item));
            writer.write_u32(price.0 as u32);
        }

        writer.into()
    }
}
pub struct PacketServerPersonalStoreTransactionUpdateMoneyAndInventory {
    pub money: Money,
    pub slot: ItemSlot,
    pub item: Option<Item>,
}

impl From<&PacketServerPersonalStoreTransactionUpdateMoneyAndInventory> for Packet {
    fn from(packet: &PacketServerPersonalStoreTransactionUpdateMoneyAndInventory) -> Self {
        let mut writer = PacketWriter::new(
            ServerPackets::PersonalStoreTransactionUpdateMoneyAndInventory as u16,
        );
        writer.write_i64(packet.money.0);
        writer.write_u8(1);
        writer.write_item_slot_u8(packet.slot);
        writer.write_item_full(packet.item.as_ref());
        writer.into()
    }
}

pub enum PacketServerPersonalStoreTransactionResult {
    Cancelled(ClientEntityId),
    SoldOut(ClientEntityId, usize, Option<Item>),
    BoughtFromStore(ClientEntityId, usize, Option<Item>),
}

impl From<&PacketServerPersonalStoreTransactionResult> for Packet {
    fn from(packet: &PacketServerPersonalStoreTransactionResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PersonalStoreTransactionResult as u16);

        match packet {
            &PacketServerPersonalStoreTransactionResult::Cancelled(store_entity_id) => {
                writer.write_entity_id(store_entity_id);
                writer.write_u8(1); // Cancelled
                writer.write_u8(0); // Update item count
            }
            PacketServerPersonalStoreTransactionResult::BoughtFromStore(
                store_entity_id,
                store_slot_index,
                store_slot,
            ) => {
                writer.write_entity_id(*store_entity_id);
                writer.write_u8(4); // Item bought from store
                writer.write_u8(1); // Update item count
                writer.write_u8(*store_slot_index as u8);
                writer.write_item_full(store_slot.as_ref());
            }
            PacketServerPersonalStoreTransactionResult::SoldOut(
                store_entity_id,
                store_slot_index,
                store_slot,
            ) => {
                writer.write_entity_id(*store_entity_id);
                writer.write_u8(2); // Sold Out
                writer.write_u8(1); // Update item count
                writer.write_u8(*store_slot_index as u8);
                writer.write_item_full(store_slot.as_ref());
            }
        }
        writer.into()
    }
}

pub struct PacketServerUseItem {
    pub entity_id: ClientEntityId,
    pub item: ItemReference,
    pub inventory_slot: ItemSlot,
}

impl From<&PacketServerUseItem> for Packet {
    fn from(packet: &PacketServerUseItem) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UseItem as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.item.item_number as u16);
        writer.write_item_slot_u8(packet.inventory_slot);
        writer.into()
    }
}

pub struct PacketServerCastSkillSelf {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub npc_motion_id: Option<usize>,
}

impl From<&PacketServerCastSkillSelf> for Packet {
    fn from(packet: &PacketServerCastSkillSelf) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillSelf as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get() as u16);
        if let Some(npc_motion_id) = packet.npc_motion_id {
            writer.write_u8(npc_motion_id as u8);
        }
        writer.into()
    }
}

pub struct PacketServerCastSkillTargetEntity {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_entity_id: ClientEntityId,
    pub target_distance: f32,
    pub target_position: Point2<f32>,
    pub npc_motion_id: Option<usize>,
}

impl From<&PacketServerCastSkillTargetEntity> for Packet {
    fn from(packet: &PacketServerCastSkillTargetEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillTargetEntity as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.target_entity_id);
        writer.write_u16(packet.skill_id.get() as u16);
        writer.write_u16(packet.target_distance as u16);
        writer.write_f32(packet.target_position.x);
        writer.write_f32(packet.target_position.y);
        if let Some(npc_motion_id) = packet.npc_motion_id {
            writer.write_u8(npc_motion_id as u8);
        }
        writer.into()
    }
}

pub struct PacketServerCastSkillTargetPosition {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_position: Point2<f32>,
    pub npc_motion_id: Option<usize>,
}

impl From<&PacketServerCastSkillTargetPosition> for Packet {
    fn from(packet: &PacketServerCastSkillTargetPosition) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillTargetPosition as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get() as u16);
        writer.write_f32(packet.target_position.x);
        writer.write_f32(packet.target_position.y);
        if let Some(npc_motion_id) = packet.npc_motion_id {
            writer.write_u8(npc_motion_id as u8);
        }
        writer.into()
    }
}

pub struct PacketServerStartCastingSkill {
    pub entity_id: ClientEntityId,
}

impl From<&PacketServerStartCastingSkill> for Packet {
    fn from(packet: &PacketServerStartCastingSkill) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::StartCastingSkill as u16);
        writer.write_entity_id(packet.entity_id);
        writer.into()
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct SkillEffectData {
    #[skip(getters)]
    skill_id: B12,
    #[skip(getters)]
    effect_success_1: bool,
    #[skip(getters)]
    effect_success_2: bool,
    #[skip(getters)]
    caster_intelligence: B10,
}

pub struct PacketServerApplySkillEffect {
    pub entity_id: ClientEntityId,
    pub caster_entity_id: ClientEntityId,
    pub caster_intelligence: i32,
    pub skill_id: SkillId,
    pub effect_success: [bool; 2],
}

impl From<&PacketServerApplySkillEffect> for Packet {
    fn from(packet: &PacketServerApplySkillEffect) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ApplySkillEffect as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.caster_entity_id);

        let data = SkillEffectData::new()
            .with_skill_id(packet.skill_id.get() as u16)
            .with_effect_success_1(packet.effect_success[0])
            .with_effect_success_2(packet.effect_success[1])
            .with_caster_intelligence(packet.caster_intelligence as u16);
        for b in data.into_bytes().iter() {
            writer.write_u8(*b);
        }

        writer.into()
    }
}

pub struct PacketServerApplySkillDamage {
    pub entity_id: ClientEntityId,
    pub caster_entity_id: ClientEntityId,
    pub caster_intelligence: i32,
    pub skill_id: SkillId,
    pub effect_success: [bool; 2],
    pub damage: Damage,
    pub is_killed: bool,
    // TODO: Optional item drop with damage
}

impl From<&PacketServerApplySkillDamage> for Packet {
    fn from(packet: &PacketServerApplySkillDamage) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ApplySkillDamage as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.caster_entity_id);

        let data = SkillEffectData::new()
            .with_skill_id(packet.skill_id.get() as u16)
            .with_effect_success_1(packet.effect_success[0])
            .with_effect_success_2(packet.effect_success[1])
            .with_caster_intelligence(packet.caster_intelligence as u16);
        for b in data.into_bytes().iter() {
            writer.write_u8(*b);
        }

        writer.write_damage_u16(&packet.damage, packet.is_killed);
        writer.into()
    }
}

pub struct PacketServerCancelCastingSkill {
    pub entity_id: ClientEntityId,
    pub reason: CancelCastingSkillReason,
}

impl From<&PacketServerCancelCastingSkill> for Packet {
    fn from(packet: &PacketServerCancelCastingSkill) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CancelCastingSkill as u16);
        writer.write_entity_id(packet.entity_id);
        match packet.reason {
            CancelCastingSkillReason::NeedAbility => writer.write_u8(1),
            CancelCastingSkillReason::NeedTarget => writer.write_u8(2),
            CancelCastingSkillReason::InvalidTarget => writer.write_u8(3),
        }
        writer.into()
    }
}

pub struct PacketServerFinishCastingSkill {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
}

impl From<&PacketServerFinishCastingSkill> for Packet {
    fn from(packet: &PacketServerFinishCastingSkill) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::FinishCastingSkill as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get() as u16);
        writer.into()
    }
}

pub struct PacketServerUpdateSpeed {
    pub entity_id: ClientEntityId,
    pub run_speed: i32,
    pub passive_attack_speed: i32,
}

impl From<&PacketServerUpdateSpeed> for Packet {
    fn from(packet: &PacketServerUpdateSpeed) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateSpeed as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.run_speed as u16);
        writer.write_u16(packet.passive_attack_speed as u16);
        writer.write_u8(0); // TODO: Weight rate
        writer.into()
    }
}

pub struct PacketServerUpdateStatusEffects<'a> {
    pub entity_id: ClientEntityId,
    pub status_effects: &'a StatusEffects,
    pub updated_hp: Option<HealthPoints>,
    pub updated_mp: Option<ManaPoints>,
}

impl<'a> From<&'a PacketServerUpdateStatusEffects<'a>> for Packet {
    fn from(packet: &'a PacketServerUpdateStatusEffects<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateStatusEffects as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_status_effects_flags_u32(packet.status_effects);

        if let Some(updated_hp) = packet.updated_hp {
            writer.write_u32(updated_hp.hp);
        }

        if let Some(updated_mp) = packet.updated_mp {
            writer.write_u32(updated_mp.mp);
        }

        writer.into()
    }
}

pub struct PacketServerMoveToggle {
    pub entity_id: ClientEntityId,
    pub move_mode: MoveMode,
    pub run_speed: Option<i32>,
}

impl From<&PacketServerMoveToggle> for Packet {
    fn from(packet: &PacketServerMoveToggle) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::MoveToggle as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_move_mode_toggle_u8(packet.move_mode);
        writer.into()
    }
}
