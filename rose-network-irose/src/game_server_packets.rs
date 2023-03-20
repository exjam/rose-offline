use bevy::math::{Vec2, Vec3};
use bitvec::array::BitArray;
use modular_bitfield::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::{num::NonZeroUsize, time::Duration};

use rose_data::{
    AbilityType, AmmoIndex, ClanMemberPosition, EquipmentIndex, EquipmentItem, Item, ItemReference,
    ItemType, MotionId, NpcId, SkillId, SkillPageType, StackableItem, VehiclePartIndex, WorldTicks,
    ZoneId,
};
use rose_data_irose::{
    decode_ability_type, decode_ammo_index, encode_ability_type, encode_ammo_index,
    IroseSkillPageType, SKILL_PAGE_SIZE,
};
use rose_game_common::{
    components::{
        ActiveQuest, BasicStatType, BasicStats, CharacterInfo, ClanLevel, ClanMark, ClanPoints,
        ClanUniqueId, DroppedItem, Equipment, ExperiencePoints, HealthPoints, Hotbar, HotbarSlot,
        Inventory, ItemSlot, Level, ManaPoints, Money, MoveMode, MoveSpeed, Npc, QuestState,
        SkillList, SkillPage, SkillPoints, SkillSlot, Stamina, StatPoints, Team, UnionMembership,
    },
    data::Damage,
    messages::{
        server::{
            ActiveStatusEffects, CancelCastingSkillReason, CharacterClanMembership,
            ClanCreateError, ClanMemberInfo, CommandState, CraftInsertGemError, LearnSkillError,
            LearnSkillSuccess, LevelUpSkillError, NpcStoreTransactionError, PartyMemberInfoOnline,
            PartyMemberLeave, PartyMemberList, PersonalStoreTransactionStatus,
            PickupItemDropContent, PickupItemDropError,
        },
        ClientEntityId, PartyItemSharing, PartyRejectInviteReason, PartyXpSharing,
    },
};
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{
    PacketEquipmentAmmoPart, PacketReadCharacterGender, PacketReadClanMark,
    PacketReadClanMemberPosition, PacketReadCommandState, PacketReadDamage, PacketReadEntityId,
    PacketReadEquipmentIndex, PacketReadHotbarSlot, PacketReadItemSlot, PacketReadItems,
    PacketReadMoveMode, PacketReadPartyMemberInfo, PacketReadPartyRules, PacketReadSkillSlot,
    PacketReadStatusEffects, PacketReadVehiclePartIndex, PacketWriteCharacterGender,
    PacketWriteClanMark, PacketWriteClanMemberPosition, PacketWriteCommandState, PacketWriteDamage,
    PacketWriteEntityId, PacketWriteEquipmentIndex, PacketWriteHotbarSlot, PacketWriteItemSlot,
    PacketWriteItems, PacketWriteMoveMode, PacketWritePartyMemberInfo, PacketWritePartyRules,
    PacketWriteSkillSlot, PacketWriteStatusEffects, PacketWriteVehiclePartIndex,
};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    AnnounceChat = 0x702,
    LogoutResult = 0x707,
    ConnectReply = 0x70c,
    SelectCharacter = 0x715,
    CharacterInventory = 0x716,
    UpdateMoneyAndInventory = 0x717,
    UpdateInventory = 0x718,
    QuestData = 0x71b,
    UpdateMoney = 0x71d,
    RewardMoney = 0x71e,
    RewardItems = 0x71f,
    UpdateAbilityValueRewardAdd = 0x720,
    UpdateAbilityValueRewardSet = 0x721,
    QuestResult = 0x730,
    RunNpcDeathTrigger = 0x731,
    JoinZone = 0x753,
    AdjustPosition = 0x770,
    ChangeNpcId = 0x774,
    UseEmote = 0x781,
    MoveToggle = 0x782,
    LocalChat = 0x783,
    Whisper = 0x784,
    ShoutChat = 0x785,
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
    NpcStoreTransactionError = 0x7a1,
    UseItem = 0x7a3,
    UpdateEquipment = 0x7a5,
    SpawnEntityItemDrop = 0x7a6,
    PickupItemDropResult = 0x7a7,
    Teleport = 0x7a8,
    UpdateBasicStat = 0x7a9,
    SetHotbarSlot = 0x7aa,
    UpdateAmmo = 0x7ab,
    BankOpen = 0x7ad,
    BankTransaction = 0x7ae,
    LearnSkillResult = 0x7b0,
    LevelUpSkillResult = 0x7b1,
    CastSkillSelf = 0x7b2,
    CastSkillTargetEntity = 0x7b3,
    CastSkillTargetPosition = 0x7b4,
    ApplySkillEffect = 0x7b5,
    ApplySkillDamage = 0x7b6,
    UpdateStatusEffects = 0x7b7,
    UpdateSpeed = 0x7b8,
    FinishCastingSkill = 0x7b9,
    StartCastingSkill = 0x7bb,
    CraftItem = 0x7bc,
    CancelCastingSkill = 0x7bd,
    OpenPersonalStore = 0x7c2,
    ClosePersonalStore = 0x7c3,
    PersonalStoreItemList = 0x7c4,
    PersonalStoreTransactionResult = 0x7c6,
    PersonalStoreTransactionUpdateMoneyAndInventory = 0x7c7,
    UpdateVehiclePart = 0x7ca,
    RepairedItemUsingItem = 0x7cb,
    RepairedItemUsingNpc = 0x7cd,
    UpdateItemLife = 0x7ce,
    PartyRequest = 0x7d0,
    PartyReply = 0x7d1,
    PartyMembers = 0x7d2,
    PartyMemberRewardItem = 0x7d3,
    PartyMemberUpdateInfo = 0x7d5,
    PartyUpdateRules = 0x7d7,
    ClanCommand = 0x7e0,
}

#[allow(dead_code)]
#[derive(Clone, Copy, FromPrimitive)]
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

impl TryFrom<&Packet> for PacketConnectionReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::ConnectReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let packet_sequence_id = reader.read_u32()?;
        let pay_flags = reader.read_u32()?;
        Ok(PacketConnectionReply {
            result,
            packet_sequence_id,
            pay_flags,
        })
    }
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

fn read_skill_page(
    reader: &mut PacketReader,
    skill_page_type: SkillPageType,
) -> Result<SkillPage, PacketError> {
    let mut skill_page = SkillPage::new(skill_page_type, SKILL_PAGE_SIZE);
    for index in 0..SKILL_PAGE_SIZE {
        skill_page.skills[index] = SkillId::new(reader.read_u16()?);
    }
    Ok(skill_page)
}

fn write_skill_page(writer: &mut PacketWriter, skill_page: Option<&SkillPage>) {
    for index in 0..SKILL_PAGE_SIZE {
        writer.write_u16(
            skill_page
                .and_then(|page| page.skills.get(index))
                .copied()
                .flatten()
                .map_or(0, |x| x.get()),
        );
    }
}

pub struct PacketServerSelectCharacter {
    pub character_info: CharacterInfo,
    pub position: Vec3,
    pub zone_id: ZoneId,
    pub equipment: Equipment,
    pub basic_stats: BasicStats,
    pub level: Level,
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

impl TryFrom<&Packet> for PacketServerSelectCharacter {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SelectCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut union_membership = UnionMembership::default();

        let mut reader = PacketReader::from(packet);
        let gender = reader.read_character_gender_u8()?;

        let zone_id = ZoneId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let revive_zone_id = ZoneId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;

        let _face = reader.read_u32()?;
        let _hair = reader.read_u32()?;
        let equipment = reader.read_equipment_visible_part()?;

        // tagBasicInfo
        let birth_stone = reader.read_u8()?;
        let face = reader.read_u8()?;
        let hair = reader.read_u8()?;
        let job = reader.read_u16()?;
        union_membership.current_union = NonZeroUsize::new(reader.read_u8()? as usize);
        let rank = reader.read_u8()?;
        let fame = reader.read_u8()?;

        // tagBasicAbility
        let strength = reader.read_u16()? as i32;
        let dexterity = reader.read_u16()? as i32;
        let intelligence = reader.read_u16()? as i32;
        let concentration = reader.read_u16()? as i32;
        let charm = reader.read_u16()? as i32;
        let sense = reader.read_u16()? as i32;

        // tagGrowAbility
        let health_points = HealthPoints::new(reader.read_u16()? as i32);
        let mana_points = ManaPoints::new(reader.read_u16()? as i32);
        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let level = Level::new(reader.read_u16()? as u32);
        let stat_points = StatPoints::new(reader.read_u16()? as u32);
        let skill_points = SkillPoints::new(reader.read_u16()? as u32);
        let _body_size = reader.read_u8()?;
        let _head_size = reader.read_u8()?;
        let _penalty_xp = reader.read_u32()?;
        let fame_g = reader.read_u16()?;
        let fame_b = reader.read_u16()?;

        for i in 0..10 {
            union_membership.points[i] = reader.read_u16()? as u32;
        }

        let _guild_id = reader.read_u32()?;
        let _guild_contribution = reader.read_u16()?;
        let _guild_pos = reader.read_u8()?;
        let _pk_flag = reader.read_u16()?;
        let stamina = Stamina::new(reader.read_u16()? as u32);

        for _ in 0..40 {
            let _seconds_remaining = reader.read_u32()?;
            let _buff_id = reader.read_u16()?;
            let _reserved = reader.read_u16()?;
        }

        // tagSkillAbility
        let basic_skill_page = read_skill_page(&mut reader, IroseSkillPageType::Basic as usize)?;
        let active_skill_page = read_skill_page(&mut reader, IroseSkillPageType::Active as usize)?;
        let passive_skill_page =
            read_skill_page(&mut reader, IroseSkillPageType::Passive as usize)?;
        let clan_skill_page = read_skill_page(&mut reader, IroseSkillPageType::Clan as usize)?;
        let skill_list = SkillList {
            pages: vec![
                basic_skill_page,
                active_skill_page,
                passive_skill_page,
                clan_skill_page,
            ],
        };

        // CHotIcons
        let mut hotbar = Hotbar::default();
        assert!(hotbar.pages.len() * hotbar.pages[0].len() == 32);
        for page in hotbar.pages.iter_mut() {
            for slot in page.iter_mut() {
                *slot = reader.read_hotbar_slot()?;
            }
        }

        let unique_id = reader.read_u32()?;
        let name = reader.read_null_terminated_utf8()?.to_string();

        Ok(PacketServerSelectCharacter {
            character_info: CharacterInfo {
                name,
                gender,
                race: 0,
                birth_stone,
                job,
                face,
                hair,
                rank,
                fame,
                fame_b,
                fame_g,
                revive_zone_id,
                revive_position: Vec3::new(0.0, 0.0, 0.0),
                unique_id,
            },
            position: Vec3::new(position_x, position_y, 0.0),
            zone_id,
            equipment,
            basic_stats: BasicStats {
                strength,
                dexterity,
                intelligence,
                concentration,
                charm,
                sense,
            },
            level,
            experience_points,
            skill_list,
            hotbar,
            health_points,
            mana_points,
            stat_points,
            skill_points,
            union_membership,
            stamina,
        })
    }
}

impl From<&PacketServerSelectCharacter> for Packet {
    fn from(packet: &PacketServerSelectCharacter) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SelectCharacter as u16);
        let character_info = &packet.character_info;
        writer.write_character_gender_u8(character_info.gender);
        writer.write_u16(packet.zone_id.get());
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        writer.write_u16(character_info.revive_zone_id.get());

        writer.write_u32(character_info.face as u32);
        writer.write_u32(character_info.hair as u32);
        writer.write_equipment_visible_part(&packet.equipment);

        // tagBasicInfo
        writer.write_u8(character_info.birth_stone);
        writer.write_u8(character_info.face);
        writer.write_u8(character_info.hair);
        writer.write_u16(character_info.job);
        writer.write_u8(
            packet
                .union_membership
                .current_union
                .map(|union| union.get())
                .unwrap_or(0) as u8,
        );
        writer.write_u8(character_info.rank);
        writer.write_u8(character_info.fame);

        // tagBasicAbility
        let basic_stats = &packet.basic_stats;
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
        write_skill_page(
            &mut writer,
            packet
                .skill_list
                .get_page(IroseSkillPageType::Basic as usize),
        );
        write_skill_page(
            &mut writer,
            packet
                .skill_list
                .get_page(IroseSkillPageType::Active as usize),
        );
        write_skill_page(
            &mut writer,
            packet
                .skill_list
                .get_page(IroseSkillPageType::Passive as usize),
        );
        write_skill_page(
            &mut writer,
            packet
                .skill_list
                .get_page(IroseSkillPageType::Clan as usize),
        );

        // CHotIcons
        assert!(packet.hotbar.pages.len() * packet.hotbar.pages[0].len() == 32);
        for page in &packet.hotbar.pages {
            for slot in page {
                writer.write_hotbar_slot(slot);
            }
        }

        writer.write_u32(character_info.unique_id);
        writer.write_null_terminated_utf8(&character_info.name);
        writer.into()
    }
}

pub struct PacketServerCharacterInventory {
    pub equipment: Equipment,
    pub inventory: Inventory,
}

impl TryFrom<&Packet> for PacketServerCharacterInventory {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::CharacterInventory as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let mut equipment = Equipment::default();
        let mut inventory = Inventory {
            money: Money(reader.read_i64()?),
            ..Default::default()
        };
        reader.read_equipment_item_full()?; // Empty item for equipment index 0

        for index in [
            EquipmentIndex::Face,
            EquipmentIndex::Head,
            EquipmentIndex::Body,
            EquipmentIndex::Back,
            EquipmentIndex::Hands,
            EquipmentIndex::Feet,
            EquipmentIndex::Weapon,
            EquipmentIndex::SubWeapon,
            EquipmentIndex::Necklace,
            EquipmentIndex::Ring,
            EquipmentIndex::Earring,
        ] {
            equipment.equipped_items[index] = reader.read_equipment_item_full()?;
        }

        for slot in inventory.equipment.slots.iter_mut() {
            *slot = reader.read_item_full()?;
        }

        for slot in inventory.consumables.slots.iter_mut() {
            *slot = reader.read_item_full()?;
        }

        for slot in inventory.materials.slots.iter_mut() {
            *slot = reader.read_item_full()?;
        }

        for slot in inventory.vehicles.slots.iter_mut() {
            *slot = reader.read_item_full()?;
        }

        for index in [AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            equipment.equipped_ammo[index] = reader.read_stackable_item_full()?;
        }

        for index in [
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Arms,
        ] {
            equipment.equipped_vehicle[index] = reader.read_equipment_item_full()?;
        }

        Ok(PacketServerCharacterInventory {
            equipment,
            inventory,
        })
    }
}

impl From<&PacketServerCharacterInventory> for Packet {
    fn from(packet: &PacketServerCharacterInventory) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CharacterInventory as u16);
        let inventory = &packet.inventory;
        let equipment = &packet.equipment;
        writer.write_i64(inventory.money.0);

        writer.write_equipment_item_full(None); // Empty item for equipment index 0
        for index in [
            EquipmentIndex::Face,
            EquipmentIndex::Head,
            EquipmentIndex::Body,
            EquipmentIndex::Back,
            EquipmentIndex::Hands,
            EquipmentIndex::Feet,
            EquipmentIndex::Weapon,
            EquipmentIndex::SubWeapon,
            EquipmentIndex::Necklace,
            EquipmentIndex::Ring,
            EquipmentIndex::Earring,
        ] {
            writer.write_equipment_item_full(equipment.get_equipment_item(index));
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

        for index in [AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            writer.write_stackable_item_full(equipment.get_ammo_item(index));
        }

        for index in [
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Arms,
        ] {
            writer.write_equipment_item_full(equipment.get_vehicle_item(index));
        }

        writer.into()
    }
}

pub struct PacketServerCharacterQuestData {
    pub quest_state: QuestState,
}

impl TryFrom<&Packet> for PacketServerCharacterQuestData {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::QuestData as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);

        let mut episode_variables: [u16; 5] = Default::default();
        let mut job_variables: [u16; 3] = Default::default();
        let mut planet_variables: [u16; 7] = Default::default();
        let mut union_variables: [u16; 10] = Default::default();
        let mut active_quests: [Option<ActiveQuest>; 10] = Default::default();
        let mut quest_switches_u32: [u32; 1024 / 32] = Default::default();

        // Episode Variables
        for variable in episode_variables.iter_mut() {
            *variable = reader.read_u16()?;
        }

        // Job Variables
        for variable in job_variables.iter_mut() {
            *variable = reader.read_u16()?;
        }

        // Planet Variables
        for variable in planet_variables.iter_mut() {
            *variable = reader.read_u16()?;
        }

        // Union Variables
        for variable in union_variables.iter_mut() {
            *variable = reader.read_u16()?;
        }

        // Active Quests
        for active_quest in active_quests.iter_mut() {
            let quest_id = reader.read_u16()? as usize;
            let expire_time = reader.read_u32()? as u64;

            let mut variables: [u16; 10] = Default::default();
            for variable in variables.iter_mut() {
                *variable = reader.read_u16()?;
            }

            let switches = reader.read_u32()?;

            let mut items: [Option<Item>; 6] = Default::default();
            for item in items.iter_mut() {
                *item = reader.read_item_full()?;
            }

            if quest_id != 0 {
                *active_quest = Some(ActiveQuest {
                    quest_id,
                    expire_time: if expire_time != 0 {
                        Some(WorldTicks(expire_time))
                    } else {
                        None
                    },
                    variables,
                    switches: BitArray::new([switches]),
                    items,
                })
            }
        }

        // Quest Switches
        for switch_u32 in quest_switches_u32.iter_mut() {
            *switch_u32 = reader.read_u32()?;
        }

        /*
        // TODO: Wish list items
        for _ in 0..30 {
            reader.read_item_full()?;
        }
        */

        Ok(PacketServerCharacterQuestData {
            quest_state: QuestState {
                episode_variables,
                job_variables,
                planet_variables,
                union_variables,
                quest_switches: BitArray::new(quest_switches_u32),
                active_quests,
            },
        })
    }
}

impl From<&PacketServerCharacterQuestData> for Packet {
    fn from(packet: &PacketServerCharacterQuestData) -> Self {
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
            writer.write_u32(quest.map_or(0, |quest| quest.switches.into_inner()[0]));

            // Active Quest Items
            for j in 0..6 {
                writer.write_item_full(
                    quest.and_then(|quest| quest.items.get(j).and_then(|item| item.as_ref())),
                );
            }
        }

        // Quest Switches
        let quest_switches_u32 = &packet.quest_state.quest_switches.into_inner();
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

impl TryFrom<&Packet> for PacketServerAttackEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::AttackEntity as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let target_entity_id = reader.read_entity_id()?;
        let distance = reader.read_u16()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_u16()?;

        Ok(Self {
            entity_id,
            target_entity_id,
            distance,
            x,
            y,
            z,
        })
    }
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
    pub is_immediate: bool,
    // TODO: Optional item drop with damage
}

impl TryFrom<&Packet> for PacketServerDamageEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::DamageEntity as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let attacker_entity_id = reader.read_entity_id()?;
        let defender_entity_id = reader.read_entity_id()?;
        let (damage, is_killed, is_immediate) = reader.read_damage_u16()?;

        Ok(Self {
            attacker_entity_id,
            defender_entity_id,
            damage,
            is_killed,
            is_immediate,
        })
    }
}

impl From<&PacketServerDamageEntity> for Packet {
    fn from(packet: &PacketServerDamageEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::DamageEntity as u16);
        writer.write_entity_id(packet.attacker_entity_id);
        writer.write_entity_id(packet.defender_entity_id);
        writer.write_damage_u16(&packet.damage, packet.is_killed, packet.is_immediate);
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

impl TryFrom<&Packet> for PacketServerMoveEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::MoveEntity as u16
            && packet.command != ServerPackets::MoveEntityWithMoveMode as u16
        {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let distance = reader.read_u16()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_u16()?;

        let move_mode = if packet.command == ServerPackets::MoveEntityWithMoveMode as u16 {
            Some(reader.read_move_mode_u8()?)
        } else {
            None
        };

        Ok(PacketServerMoveEntity {
            entity_id,
            target_entity_id,
            distance,
            x,
            y,
            z,
            move_mode,
        })
    }
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

pub struct PacketServerJoinZone {
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

impl TryFrom<&Packet> for PacketServerJoinZone {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::JoinZone as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = ClientEntityId(reader.read_u16()? as usize);
        let health_points = HealthPoints::new(reader.read_u16()? as i32);
        let mana_points = ManaPoints::new(reader.read_u16()? as i32);

        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let _penalty_xp = reader.read_u32();

        // tagVAR_GLOBAL
        let craft_rate = reader.read_u16()? as i32;
        let _update_time = reader.read_u32()?;
        let world_price_rate = reader.read_u16()? as i32;
        let town_price_rate = reader.read_u8()? as i32;
        let item_price_rate_0 = reader.read_u8()? as i32;
        for _ in 1..11 {
            let _item_price_rate_n = reader.read_u8()?;
        }
        let _global_flags = reader.read_u32()?;

        let world_ticks = WorldTicks(reader.read_u32()? as u64);
        let team = Team::new(reader.read_u32()?);

        Ok(PacketServerJoinZone {
            entity_id,
            experience_points,
            team,
            health_points,
            mana_points,
            world_ticks,
            craft_rate,
            world_price_rate,
            town_price_rate,
            item_price_rate: item_price_rate_0,
        })
    }
}

impl From<&PacketServerJoinZone> for Packet {
    fn from(packet: &PacketServerJoinZone) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::JoinZone as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.health_points.hp as u16);
        writer.write_u16(packet.mana_points.mp as u16);

        writer.write_u32(packet.experience_points.xp as u32);
        writer.write_u32(0); // penalty xp

        // tagVAR_GLOBAL
        writer.write_u16(packet.craft_rate as u16);
        writer.write_u32(0); // update time
        writer.write_u16(packet.world_price_rate as u16);
        writer.write_u8(packet.town_price_rate as u8);
        for _ in 0..11 {
            writer.write_u8(packet.item_price_rate as u8);
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

impl<'a> TryFrom<&'a Packet> for PacketServerLocalChat<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::LocalChat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketServerLocalChat { entity_id, text })
    }
}

impl<'a> From<&'a PacketServerLocalChat<'a>> for Packet {
    fn from(packet: &'a PacketServerLocalChat<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LocalChat as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_null_terminated_utf8(packet.text);
        writer.into()
    }
}

pub struct PacketServerShoutChat<'a> {
    pub name: &'a str,
    pub text: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketServerShoutChat<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::ShoutChat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let name = reader.read_null_terminated_utf8()?;
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketServerShoutChat { name, text })
    }
}

impl<'a> From<&'a PacketServerShoutChat<'a>> for Packet {
    fn from(packet: &'a PacketServerShoutChat<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ShoutChat as u16);
        writer.write_null_terminated_utf8(packet.name);
        writer.write_null_terminated_utf8(packet.text);
        writer.into()
    }
}

pub struct PacketServerAnnounceChat<'a> {
    pub name: Option<&'a str>,
    pub text: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketServerAnnounceChat<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::AnnounceChat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let text = reader.read_null_terminated_utf8()?;
        let name = reader.read_null_terminated_utf8().ok();
        Ok(PacketServerAnnounceChat { name, text })
    }
}

impl<'a> From<&'a PacketServerAnnounceChat<'a>> for Packet {
    fn from(packet: &'a PacketServerAnnounceChat<'a>) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::AnnounceChat as u16);
        writer.write_null_terminated_utf8(packet.text);
        if let Some(name) = packet.name {
            writer.write_null_terminated_utf8(name);
        }
        writer.into()
    }
}

pub struct PacketServerWhisper<'a> {
    pub from: &'a str,
    pub text: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketServerWhisper<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::Whisper as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let from = reader.read_null_terminated_utf8()?;
        let text = reader.read_null_terminated_utf8()?;
        Ok(PacketServerWhisper { from, text })
    }
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

impl TryFrom<&Packet> for PacketServerStopMoveEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::StopMoveEntity as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_u16()?;
        Ok(Self { entity_id, x, y, z })
    }
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

impl TryFrom<&Packet> for PacketServerTeleport {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::Teleport as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let zone_id = ZoneId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let run_mode = reader.read_u8()?;
        let ride_mode = reader.read_u8()?;
        Ok(PacketServerTeleport {
            entity_id,
            zone_id,
            x,
            y,
            run_mode,
            ride_mode,
        })
    }
}

impl From<&PacketServerTeleport> for Packet {
    fn from(packet: &PacketServerTeleport) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::Teleport as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.zone_id.get());
        writer.write_f32(packet.x);
        writer.write_f32(packet.y);
        writer.write_u8(packet.run_mode);
        writer.write_u8(packet.ride_mode);
        writer.into()
    }
}

#[derive(Debug)]
pub struct PacketServerSetHotbarSlot {
    pub slot_index: usize,
    pub slot: Option<HotbarSlot>,
}

impl TryFrom<&Packet> for PacketServerSetHotbarSlot {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::SetHotbarSlot as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let slot_index = reader.read_u8()? as usize;
        let slot = reader.read_hotbar_slot()?;

        Ok(Self { slot_index, slot })
    }
}

impl From<&PacketServerSetHotbarSlot> for Packet {
    fn from(packet: &PacketServerSetHotbarSlot) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SetHotbarSlot as u16);
        writer.write_u8(packet.slot_index as u8);
        writer.write_hotbar_slot(&packet.slot);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityItemDrop {
    pub entity_id: ClientEntityId,
    pub dropped_item: DroppedItem,
    pub position: Vec3,
    pub owner_entity_id: Option<ClientEntityId>,
    pub remaining_time: Duration,
}

impl TryFrom<&Packet> for PacketServerSpawnEntityItemDrop {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SpawnEntityItemDrop as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let position = Vec3::new(position_x, position_y, 0.0);

        let (item, money) = reader.read_item_or_money_full()?;
        let dropped_item = if let Some(item) = item {
            DroppedItem::Item(item)
        } else if let Some(money) = money {
            DroppedItem::Money(money)
        } else {
            return Err(PacketError::InvalidPacket);
        };

        let entity_id = reader.read_entity_id()?;
        let owner_entity_id = reader.read_option_entity_id()?;
        let remaining_time = Duration::from_millis(reader.read_u16()? as u64);

        Ok(Self {
            entity_id,
            dropped_item,
            position,
            owner_entity_id,
            remaining_time,
        })
    }
}

impl From<&PacketServerSpawnEntityItemDrop> for Packet {
    fn from(packet: &PacketServerSpawnEntityItemDrop) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityItemDrop as u16);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        match &packet.dropped_item {
            DroppedItem::Item(item) => writer.write_item_full(Some(item)),
            DroppedItem::Money(amount) => writer.write_item_full_money(*amount),
        }
        writer.write_entity_id(packet.entity_id);
        writer.write_option_entity_id(packet.owner_entity_id);
        writer.write_u16(packet.remaining_time.as_millis() as u16);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityNpc {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub direction: f32,
    pub position: Vec3,
    pub team: Team,
    pub destination: Option<Vec3>,
    pub command: CommandState,
    pub target_entity_id: Option<ClientEntityId>,
    pub health: HealthPoints,
    pub move_mode: MoveMode,
    pub status_effects: ActiveStatusEffects,
}

impl TryFrom<&Packet> for PacketServerSpawnEntityNpc {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SpawnEntityNpc as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let position = Vec3::new(position_x, position_y, 0.0);
        let destination_x = reader.read_f32()?;
        let destination_y = reader.read_f32()?;
        let destination = if destination_x != 0.0 && destination_y != 0.0 {
            Some(Vec3::new(destination_x, destination_y, 0.0))
        } else {
            None
        };
        let command = reader.read_command_state()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let move_mode = reader.read_move_mode_u8()?;
        let health = HealthPoints::new(reader.read_i32()?);
        let team = Team::new(reader.read_u32()?);
        let mut status_effects = ActiveStatusEffects::default();
        reader.read_status_effects_flags_u32(&mut status_effects)?;
        let npc_id = NpcId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let quest_index = reader.read_u16()?;
        let direction = reader.read_f32()?;
        let _event_status = reader.read_u16()?;
        reader.read_status_effects_values(&mut status_effects)?;
        Ok(PacketServerSpawnEntityNpc {
            entity_id,
            npc: Npc::new(npc_id, quest_index),
            direction,
            position,
            team,
            destination,
            command,
            target_entity_id,
            health,
            move_mode,
            status_effects,
        })
    }
}

impl From<&PacketServerSpawnEntityNpc> for Packet {
    fn from(packet: &PacketServerSpawnEntityNpc) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityNpc as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        if let Some(destination) = packet.destination.as_ref() {
            writer.write_f32(destination.x);
            writer.write_f32(destination.y);
        } else {
            writer.write_f32(0.0);
            writer.write_f32(0.0);
        }
        writer.write_command_state(&packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_i32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(&packet.status_effects);
        writer.write_u16(packet.npc.id.get());
        writer.write_u16(packet.npc.quest_index);
        writer.write_f32(packet.direction);
        writer.write_u16(0); // event status
        writer.write_status_effects_values(&packet.status_effects);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityMonster {
    pub entity_id: ClientEntityId,
    pub npc: Npc,
    pub position: Vec3,
    pub destination: Option<Vec3>,
    pub team: Team,
    pub health: HealthPoints,
    pub command: CommandState,
    pub target_entity_id: Option<ClientEntityId>,
    pub move_mode: MoveMode,
    pub status_effects: ActiveStatusEffects,
}

impl TryFrom<&Packet> for PacketServerSpawnEntityMonster {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SpawnEntityMonster as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let position = Vec3::new(position_x, position_y, 0.0);
        let destination_x = reader.read_f32()?;
        let destination_y = reader.read_f32()?;
        let destination = if destination_x != 0.0 && destination_y != 0.0 {
            Some(Vec3::new(destination_x, destination_y, 0.0))
        } else {
            None
        };
        let command = reader.read_command_state()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let move_mode = reader.read_move_mode_u8()?;
        let health = HealthPoints::new(reader.read_i32()?);
        let team = Team::new(reader.read_u32()?);
        let mut status_effects = ActiveStatusEffects::default();
        reader.read_status_effects_flags_u32(&mut status_effects)?;
        let npc_id = NpcId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let quest_index = reader.read_u16()?;
        reader.read_status_effects_values(&mut status_effects)?;
        Ok(PacketServerSpawnEntityMonster {
            entity_id,
            npc: Npc::new(npc_id, quest_index),
            position,
            team,
            destination,
            command,
            target_entity_id,
            health,
            move_mode,
            status_effects,
        })
    }
}

impl From<&PacketServerSpawnEntityMonster> for Packet {
    fn from(packet: &PacketServerSpawnEntityMonster) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityMonster as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        if let Some(destination) = packet.destination.as_ref() {
            writer.write_f32(destination.x);
            writer.write_f32(destination.y);
        } else {
            writer.write_f32(0.0);
            writer.write_f32(0.0);
        }
        writer.write_command_state(&packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_i32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(&packet.status_effects);
        writer.write_u16(packet.npc.id.get());
        writer.write_u16(packet.npc.quest_index);
        writer.write_status_effects_values(&packet.status_effects);
        writer.into()
    }
}

pub struct PacketServerSpawnEntityCharacter {
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

impl TryFrom<&Packet> for PacketServerSpawnEntityCharacter {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::SpawnEntityCharacter as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let destination_x = reader.read_f32()?;
        let destination_y = reader.read_f32()?;
        let destination = if destination_x != 0.0 && destination_y != 0.0 {
            Some(Vec3::new(destination_x, destination_y, 0.0))
        } else {
            None
        };
        let command = reader.read_command_state()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let move_mode = reader.read_move_mode_u8()?;
        let health = HealthPoints::new(reader.read_i32()?);
        let team = Team::new(reader.read_u32()?);
        let mut status_effects = ActiveStatusEffects::default();
        reader.read_status_effects_flags_u32(&mut status_effects)?;

        let gender = reader.read_character_gender_u8()?;
        let move_speed = MoveSpeed::new(reader.read_u16()? as f32);
        let passive_attack_speed = reader.read_u16()? as i32;
        let _weight_rate = reader.read_u8()?;
        let face = reader.read_u32()? as u8;
        let hair = reader.read_u32()? as u8;
        let mut equipment = reader.read_equipment_visible_part()?;

        for index in [AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            equipment.equipped_ammo[index] = reader.read_equipment_ammo_part()?;
        }

        let job = reader.read_u16()?;
        let level = Level::new(reader.read_u8()? as u32);

        for index in [
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Arms,
        ] {
            equipment.equipped_vehicle[index] =
                reader.read_equipment_item_part(ItemType::Weapon)?;
        }

        let position_z = reader.read_u16()? as f32;
        let sub_flags = reader.read_u32()?; // TODO: Use sub flags
        let name = reader.read_null_terminated_utf8()?.to_string();
        reader.read_status_effects_values(&mut status_effects)?;

        let personal_store_info = if sub_flags & 2 != 0 {
            let store_skin = reader.read_u16()? as i32;
            let store_title = reader.read_null_terminated_utf8()?.to_string();
            Some((store_skin, store_title))
        } else {
            None
        };

        let clan_membership = |reader: &mut PacketReader| -> Option<CharacterClanMembership> {
            let clan_unique_id = ClanUniqueId::new(reader.read_u32().ok()?)?;
            let mark = reader.read_clan_mark_u32().ok()?;
            let level = ClanLevel::new(reader.read_u8().ok()? as u32)?;
            let position = reader.read_clan_member_position_u8().ok()?;
            let name = reader.read_null_terminated_utf8().ok()?.to_string();

            Some(CharacterClanMembership {
                clan_unique_id,
                mark,
                level,
                name,
                position,
            })
        }(&mut reader);

        Ok(Self {
            entity_id,
            position: Vec3::new(position_x, position_y, position_z),
            team,
            destination,
            command,
            target_entity_id,
            health,
            move_mode,
            status_effects,
            character_info: CharacterInfo {
                name,
                gender,
                race: 0,
                birth_stone: 0,
                job,
                face,
                hair,
                rank: 0,
                fame: 0,
                fame_b: 0,
                fame_g: 0,
                revive_zone_id: ZoneId::new(1).unwrap(),
                revive_position: Vec3::new(0.0, 0.0, 0.0),
                unique_id: 0,
            },
            equipment,
            level,
            move_speed,
            passive_attack_speed,
            personal_store_info,
            clan_membership,
        })
    }
}

impl From<&PacketServerSpawnEntityCharacter> for Packet {
    fn from(packet: &PacketServerSpawnEntityCharacter) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::SpawnEntityCharacter as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        if let Some(destination) = packet.destination.as_ref() {
            writer.write_f32(destination.x);
            writer.write_f32(destination.y);
        } else {
            writer.write_f32(0.0);
            writer.write_f32(0.0);
        }
        writer.write_command_state(&packet.command);
        writer.write_option_entity_id(packet.target_entity_id);
        writer.write_move_mode_u8(packet.move_mode);
        writer.write_i32(packet.health.hp);
        writer.write_u32(packet.team.id);
        writer.write_status_effects_flags_u32(&packet.status_effects);
        writer.write_character_gender_u8(packet.character_info.gender);
        writer.write_u16(packet.move_speed.speed as u16);
        writer.write_u16(packet.passive_attack_speed as u16);
        writer.write_u8(0); // TODO: Weight rate

        writer.write_u32(packet.character_info.face as u32);
        writer.write_u32(packet.character_info.hair as u32);
        writer.write_equipment_visible_part(&packet.equipment);

        for index in &[AmmoIndex::Arrow, AmmoIndex::Bullet, AmmoIndex::Throw] {
            writer.write_equipment_ammo_part(packet.equipment.get_ammo_item(*index));
        }

        writer.write_u16(packet.character_info.job);
        writer.write_u8(packet.level.level as u8);

        for index in &[
            VehiclePartIndex::Body,
            VehiclePartIndex::Engine,
            VehiclePartIndex::Leg,
            VehiclePartIndex::Arms,
        ] {
            writer.write_equipment_item_part(packet.equipment.get_vehicle_item(*index));
        }

        writer.write_u16(packet.position.z as u16);

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

        writer.write_status_effects_values(&packet.status_effects);

        if let Some((personal_store_skin, personal_store_title)) =
            packet.personal_store_info.as_ref()
        {
            writer.write_u16(*personal_store_skin as u16);
            writer.write_null_terminated_utf8(personal_store_title);
        }

        if let Some(clan_membership) = packet.clan_membership.as_ref() {
            writer.write_u32(clan_membership.clan_unique_id.get());
            writer.write_clan_mark_u32(&clan_membership.mark);
            writer.write_u8(clan_membership.level.get() as u8);
            writer.write_clan_member_position_u8(&clan_membership.position);
            writer.write_null_terminated_utf8(&clan_membership.name);
        }

        writer.into()
    }
}

pub struct PacketServerRemoveEntities {
    pub entity_ids: Vec<ClientEntityId>,
}

impl TryFrom<&Packet> for PacketServerRemoveEntities {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::RemoveEntities as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let mut entity_ids = Vec::new();
        while let Ok(entity_id) = reader.read_entity_id() {
            entity_ids.push(entity_id);
        }
        Ok(PacketServerRemoveEntities { entity_ids })
    }
}

impl From<&PacketServerRemoveEntities> for Packet {
    fn from(packet: &PacketServerRemoveEntities) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RemoveEntities as u16);
        for entity_id in packet.entity_ids.iter() {
            writer.write_entity_id(*entity_id);
        }
        writer.into()
    }
}

pub struct PacketServerUpdateInventory {
    pub items: Vec<(ItemSlot, Option<Item>)>,
    pub with_money: Option<Money>,
}

impl TryFrom<&Packet> for PacketServerUpdateInventory {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateInventory as u16
            && packet.command != ServerPackets::UpdateMoneyAndInventory as u16
        {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let with_money = if packet.command == ServerPackets::UpdateMoneyAndInventory as u16 {
            Some(Money(reader.read_i64()?))
        } else {
            None
        };

        let num_items = reader.read_u8()? as usize;
        let mut items = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            let item_slot = reader.read_item_slot_u8()?;
            let item = reader.read_item_full()?;
            items.push((item_slot, item));
        }

        Ok(Self { items, with_money })
    }
}

impl From<&PacketServerUpdateInventory> for Packet {
    fn from(packet: &PacketServerUpdateInventory) -> Self {
        let command = if packet.with_money.is_some() {
            ServerPackets::UpdateMoneyAndInventory
        } else {
            ServerPackets::UpdateInventory
        };
        let mut writer = PacketWriter::new(command as u16);

        if let Some(money) = packet.with_money {
            writer.write_i64(money.0);
        }

        writer.write_u8(packet.items.len() as u8);
        for (slot, item) in packet.items.iter() {
            writer.write_item_slot_u8(*slot);
            writer.write_item_full(item.as_ref());
        }
        writer.into()
    }
}

pub struct PacketServerUpdateMoney {
    pub money: Money,
}

impl TryFrom<&Packet> for PacketServerUpdateMoney {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateMoney as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let money = Money(reader.read_i64()?);
        Ok(Self { money })
    }
}

impl From<&PacketServerUpdateMoney> for Packet {
    fn from(packet: &PacketServerUpdateMoney) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateMoney as u16);
        writer.write_i64(packet.money.0);
        writer.into()
    }
}

pub struct PacketServerRewardItems {
    pub items: Vec<(ItemSlot, Option<Item>)>,
}

impl TryFrom<&Packet> for PacketServerRewardItems {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::RewardItems as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_count = reader.read_u8()? as usize;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let slot = reader.read_item_slot_u8()?;
            let item = reader.read_item_full()?;
            items.push((slot, item));
        }
        Ok(Self { items })
    }
}

impl From<&PacketServerRewardItems> for Packet {
    fn from(packet: &PacketServerRewardItems) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RewardItems as u16);
        writer.write_u8(packet.items.len() as u8);
        for (slot, item) in packet.items.iter() {
            writer.write_item_slot_u8(*slot);
            writer.write_item_full(item.as_ref());
        }
        writer.into()
    }
}

pub struct PacketServerRewardMoney {
    pub money: Money,
}

impl TryFrom<&Packet> for PacketServerRewardMoney {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::RewardMoney as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let money = Money(reader.read_i64()?);
        Ok(Self { money })
    }
}

impl From<&PacketServerRewardMoney> for Packet {
    fn from(packet: &PacketServerRewardMoney) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RewardMoney as u16);
        writer.write_i64(packet.money.0);
        writer.into()
    }
}

pub struct PacketServerUpdateAmmo {
    pub entity_id: ClientEntityId,
    pub ammo_index: AmmoIndex,
    pub item: Option<StackableItem>,
}

impl TryFrom<&Packet> for PacketServerUpdateAmmo {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateAmmo as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;

        let ammo_part = PacketEquipmentAmmoPart::from_bytes(
            reader.read_fixed_length_bytes(2)?.try_into().unwrap(),
        );

        let ammo_index =
            decode_ammo_index(ammo_part.item_type() as usize).ok_or(PacketError::InvalidPacket)?;

        let item = StackableItem::new(
            ItemReference::new(ItemType::Material, ammo_part.item_number() as usize),
            999,
        );

        Ok(Self {
            entity_id,
            ammo_index,
            item,
        })
    }
}

impl From<&PacketServerUpdateAmmo> for Packet {
    fn from(packet: &PacketServerUpdateAmmo) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateAmmo as u16);
        writer.write_entity_id(packet.entity_id);

        let part = PacketEquipmentAmmoPart::new()
            .with_item_number(
                packet
                    .item
                    .as_ref()
                    .map(|item| item.item.item_number)
                    .unwrap_or(0) as u16,
            )
            .with_item_type(encode_ammo_index(packet.ammo_index).unwrap_or(0) as u8);
        for b in part.into_bytes().iter() {
            writer.write_u8(*b);
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

impl TryFrom<&Packet> for PacketServerUpdateEquipment {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateEquipment as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let equipment_index = reader.read_equipment_index_u16()?;
        let item_type = match equipment_index {
            EquipmentIndex::Face => ItemType::Face,
            EquipmentIndex::Head => ItemType::Head,
            EquipmentIndex::Body => ItemType::Body,
            EquipmentIndex::Back => ItemType::Back,
            EquipmentIndex::Hands => ItemType::Hands,
            EquipmentIndex::Feet => ItemType::Feet,
            EquipmentIndex::Weapon => ItemType::Weapon,
            EquipmentIndex::SubWeapon => ItemType::SubWeapon,
            EquipmentIndex::Necklace | EquipmentIndex::Ring | EquipmentIndex::Earring => {
                ItemType::Jewellery
            }
        };
        let item = reader.read_equipment_item_part(item_type)?;
        let run_speed = reader.read_u16().ok();

        Ok(Self {
            entity_id,
            equipment_index,
            item,
            run_speed,
        })
    }
}

impl From<&PacketServerUpdateEquipment> for Packet {
    fn from(packet: &PacketServerUpdateEquipment) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateEquipment as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_equipment_index_u16(packet.equipment_index);
        writer.write_equipment_item_part(packet.item.as_ref());
        if let Some(run_speed) = packet.run_speed {
            writer.write_u16(run_speed);
        }
        writer.into()
    }
}

pub struct PacketServerUpdateVehiclePart {
    pub entity_id: ClientEntityId,
    pub vehicle_part_index: VehiclePartIndex,
    pub item: Option<EquipmentItem>,
    pub run_speed: Option<u16>,
}

impl TryFrom<&Packet> for PacketServerUpdateVehiclePart {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateVehiclePart as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let vehicle_part_index = reader.read_vehicle_part_index_u16()?;
        let item = reader.read_equipment_item_part(ItemType::Vehicle)?;
        let run_speed = reader.read_u16().ok();

        Ok(Self {
            entity_id,
            vehicle_part_index,
            item,
            run_speed,
        })
    }
}

impl From<&PacketServerUpdateVehiclePart> for Packet {
    fn from(packet: &PacketServerUpdateVehiclePart) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateVehiclePart as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_vehicle_part_index_u16(packet.vehicle_part_index);
        writer.write_equipment_item_part(packet.item.as_ref());
        if let Some(run_speed) = packet.run_speed {
            writer.write_u16(run_speed);
        }
        writer.into()
    }
}

pub struct PacketServerUpdateItemLife {
    pub item_slot: ItemSlot,
    pub life: u16,
}

impl TryFrom<&Packet> for PacketServerUpdateItemLife {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateItemLife as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_slot = reader.read_item_slot_u16()?;
        let life = reader.read_u16()?;

        Ok(Self { item_slot, life })
    }
}

impl From<&PacketServerUpdateItemLife> for Packet {
    fn from(packet: &PacketServerUpdateItemLife) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateItemLife as u16);
        writer.write_item_slot_u16(packet.item_slot);
        writer.write_u16(packet.life);
        writer.into()
    }
}

pub struct PacketServerUpdateLevel {
    pub entity_id: ClientEntityId,
    pub update_values: Option<(Level, ExperiencePoints, StatPoints, SkillPoints)>,
}

impl TryFrom<&Packet> for PacketServerUpdateLevel {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateLevel as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let update_values = if let Ok(level) = reader.read_u16() {
            let level = Level::new(level as u32);
            let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
            let stat_points = StatPoints::new(reader.read_u16()? as u32);
            let skill_points = SkillPoints::new(reader.read_u16()? as u32);
            Some((level, experience_points, stat_points, skill_points))
        } else {
            None
        };

        Ok(Self {
            entity_id,
            update_values,
        })
    }
}

impl From<&PacketServerUpdateLevel> for Packet {
    fn from(packet: &PacketServerUpdateLevel) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateLevel as u16);
        writer.write_entity_id(packet.entity_id);
        if let Some((level, experience_points, stat_points, skill_points)) = &packet.update_values {
            writer.write_u16(level.level as u16);
            writer.write_u32(experience_points.xp as u32);
            writer.write_u16(stat_points.points as u16);
            writer.write_u16(skill_points.points as u16);
        }
        writer.into()
    }
}

pub struct PacketServerUpdateXpStamina {
    pub xp: u64,
    pub stamina: u32,
    pub source_entity_id: Option<ClientEntityId>,
}

impl TryFrom<&Packet> for PacketServerUpdateXpStamina {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateXpStamina as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let xp = reader.read_u32()? as u64;
        let stamina = reader.read_u16()? as u32;
        let source_entity_id = reader.read_option_entity_id()?;

        Ok(Self {
            xp,
            stamina,
            source_entity_id,
        })
    }
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

impl TryFrom<&Packet> for PacketServerUpdateBasicStat {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateBasicStat as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let basic_stat_type = match reader.read_u8()? {
            0 => BasicStatType::Strength,
            1 => BasicStatType::Dexterity,
            2 => BasicStatType::Intelligence,
            3 => BasicStatType::Concentration,
            4 => BasicStatType::Charm,
            5 => BasicStatType::Sense,
            _ => return Err(PacketError::InvalidPacket),
        };
        let value = reader.read_u16()? as i32;

        Ok(Self {
            basic_stat_type,
            value,
        })
    }
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

pub struct PacketServerPickupItemDropResult {
    pub item_entity_id: ClientEntityId,
    pub result: Result<PickupItemDropContent, PickupItemDropError>,
}

impl TryFrom<&Packet> for PacketServerPickupItemDropResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PickupItemDropResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let item_entity_id = reader.read_entity_id()?;
        let result = match reader.read_u8()? {
            0 => {
                let item_slot = reader.read_item_slot_u16().ok();
                match reader.read_item_or_money_full()? {
                    (None, Some(money)) => Ok(PickupItemDropContent::Money(money)),
                    (Some(item), None) => Ok(PickupItemDropContent::Item(
                        item_slot.ok_or(PacketError::InvalidPacket)?,
                        item,
                    )),
                    _ => return Err(PacketError::InvalidPacket),
                }
            }
            1 => Err(PickupItemDropError::NotExist),
            2 => Err(PickupItemDropError::NoPermission),
            3 => Err(PickupItemDropError::InventoryFull),
            _ => Err(PickupItemDropError::NotExist),
        };

        Ok(Self {
            item_entity_id,
            result,
        })
    }
}

impl From<&PacketServerPickupItemDropResult> for Packet {
    fn from(packet: &PacketServerPickupItemDropResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PickupItemDropResult as u16);
        writer.write_entity_id(packet.item_entity_id);
        match &packet.result {
            Ok(PickupItemDropContent::Item(slot, item)) => {
                writer.write_u8(0); // OK
                writer.write_item_slot_u16(*slot);
                writer.write_item_full(Some(item));
            }
            Ok(PickupItemDropContent::Money(money)) => {
                writer.write_u8(0); // OK
                writer.write_u16(0); // Slot
                writer.write_item_full_money(*money);
            }
            Err(error) => {
                match error {
                    PickupItemDropError::NotExist => writer.write_u8(1),
                    PickupItemDropError::NoPermission => writer.write_u8(2),
                    PickupItemDropError::InventoryFull => writer.write_u8(3),
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

impl TryFrom<&Packet> for PacketServerLogoutResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::LogoutResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let duration = reader.read_u16()?;
        if duration == 0 {
            Ok(Self { result: Ok(()) })
        } else {
            Ok(Self {
                result: Err(Duration::from_secs(duration as u64)),
            })
        }
    }
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

impl TryFrom<&Packet> for PacketServerQuestResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::QuestResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = match reader.read_u8()? {
            1 => PacketServerQuestResultType::AddSuccess,
            2 => PacketServerQuestResultType::AddFailed,
            3 => PacketServerQuestResultType::DeleteSuccess,
            4 => PacketServerQuestResultType::DeleteFailed,
            5 => PacketServerQuestResultType::TriggerSuccess,
            6 => PacketServerQuestResultType::TriggerFailed,
            _ => return Err(PacketError::InvalidPacket),
        };
        let slot = reader.read_u8()?;
        let quest_id = reader.read_u32()?;

        Ok(Self {
            result,
            slot,
            quest_id,
        })
    }
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

impl TryFrom<&Packet> for PacketServerUpdateAbilityValue {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateAbilityValueRewardAdd as u16
            && packet.command != ServerPackets::UpdateAbilityValueRewardSet as u16
        {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let ability_type =
            decode_ability_type(reader.read_u16()? as usize).ok_or(PacketError::InvalidPacket)?;
        let value = reader.read_i32()?;

        Ok(Self {
            is_add: packet.command == ServerPackets::UpdateAbilityValueRewardAdd as u16,
            ability_type,
            value,
        })
    }
}

impl From<&PacketServerUpdateAbilityValue> for Packet {
    fn from(packet: &PacketServerUpdateAbilityValue) -> Self {
        let command = if packet.is_add {
            ServerPackets::UpdateAbilityValueRewardAdd
        } else {
            ServerPackets::UpdateAbilityValueRewardSet
        };

        let mut writer = PacketWriter::new(command as u16);
        writer.write_u16(encode_ability_type(packet.ability_type).unwrap_or(0) as u16);
        writer.write_i32(packet.value);
        writer.into()
    }
}

pub struct PacketServerLearnSkillResult {
    pub result: Result<LearnSkillSuccess, LearnSkillError>,
}

impl TryFrom<&Packet> for PacketServerLearnSkillResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::LearnSkillResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let result = match reader.read_u8()? {
            0 => Err(LearnSkillError::AlreadyLearnt),
            1 => {
                let skill_slot = reader.read_skill_slot_u8()?;
                let skill_id = SkillId::new(reader.read_u16()?);
                let updated_skill_points = SkillPoints::new(reader.read_u16()? as u32);
                Ok(LearnSkillSuccess {
                    skill_slot,
                    skill_id,
                    updated_skill_points,
                })
            }
            2 => Err(LearnSkillError::JobRequirement),
            3 => Err(LearnSkillError::SkillRequirement),
            4 => Err(LearnSkillError::AbilityRequirement),
            5 => Err(LearnSkillError::Full),
            6 => Err(LearnSkillError::InvalidSkillId),
            7 => Err(LearnSkillError::SkillPointRequirement),
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(Self { result })
    }
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
                writer.write_u16(skill_id.map_or(0, |skill_id| skill_id.get()));
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

pub struct PacketServerLevelUpSkillResult {
    pub result: Result<(SkillSlot, SkillId), LevelUpSkillError>,
    pub updated_skill_points: SkillPoints,
}

impl TryFrom<&Packet> for PacketServerLevelUpSkillResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::LevelUpSkillResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let skill_slot = reader.read_skill_slot_u8()?;
        let skill_id = SkillId::new(reader.read_u16()?);
        let updated_skill_points = SkillPoints::new(reader.read_u16()? as u32);

        let result = match reader.read_u8()? {
            0 => Ok((skill_slot, skill_id.ok_or(PacketError::InvalidPacket)?)),
            1 => Err(LevelUpSkillError::Failed),
            2 => Err(LevelUpSkillError::SkillPointRequirement),
            3 => Err(LevelUpSkillError::AbilityRequirement),
            4 => Err(LevelUpSkillError::JobRequirement),
            5 => Err(LevelUpSkillError::SkillRequirement),
            6 => Err(LevelUpSkillError::MoneyRequirement),
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(Self {
            result,
            updated_skill_points,
        })
    }
}

impl From<&PacketServerLevelUpSkillResult> for Packet {
    fn from(packet: &PacketServerLevelUpSkillResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::LevelUpSkillResult as u16);
        match packet.result {
            Ok((skill_slot, skill_id)) => {
                writer.write_u8(0); // Success
                writer.write_skill_slot_u8(skill_slot);
                writer.write_u16(skill_id.get());
            }
            Err(error) => {
                match error {
                    LevelUpSkillError::Failed => writer.write_u8(1),
                    LevelUpSkillError::SkillPointRequirement => writer.write_u8(2),
                    LevelUpSkillError::AbilityRequirement => writer.write_u8(3),
                    LevelUpSkillError::JobRequirement => writer.write_u8(4),
                    LevelUpSkillError::SkillRequirement => writer.write_u8(5),
                    LevelUpSkillError::MoneyRequirement => writer.write_u8(6),
                }
                writer.write_u8(0); // Slot
                writer.write_u16(0); // Index
            }
        }
        writer.write_u16(packet.updated_skill_points.points as u16);
        writer.into()
    }
}

pub struct PacketServerRunNpcDeathTrigger {
    pub npc_id: NpcId,
}

impl TryFrom<&Packet> for PacketServerRunNpcDeathTrigger {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::RunNpcDeathTrigger as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let npc_id = NpcId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;

        Ok(Self { npc_id })
    }
}

impl From<&PacketServerRunNpcDeathTrigger> for Packet {
    fn from(packet: &PacketServerRunNpcDeathTrigger) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RunNpcDeathTrigger as u16);
        writer.write_u16(packet.npc_id.get());
        writer.into()
    }
}

pub struct PacketServerOpenPersonalStore<'a> {
    pub entity_id: ClientEntityId,
    pub skin: i32,
    pub title: &'a str,
}

impl<'a> TryFrom<&'a Packet> for PacketServerOpenPersonalStore<'a> {
    type Error = PacketError;

    fn try_from(packet: &'a Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::OpenPersonalStore as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let skin = reader.read_u16()? as i32;
        let title = reader.read_null_terminated_utf8()?;
        Ok(Self {
            entity_id,
            skin,
            title,
        })
    }
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

pub struct PacketServerClosePersonalStore {
    pub entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketServerClosePersonalStore {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::ClosePersonalStore as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;

        Ok(Self { entity_id })
    }
}

impl From<&PacketServerClosePersonalStore> for Packet {
    fn from(packet: &PacketServerClosePersonalStore) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ClosePersonalStore as u16);
        writer.write_entity_id(packet.entity_id);
        writer.into()
    }
}

pub struct PacketServerPersonalStoreItemList {
    pub sell_items: Vec<(u8, Item, Money)>,
    pub buy_items: Vec<(u8, Item, Money)>,
}

impl TryFrom<&Packet> for PacketServerPersonalStoreItemList {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PersonalStoreItemList as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let num_sell_items = reader.read_u8()? as usize;
        let num_buy_items = reader.read_u8()? as usize;
        let mut sell_items = Vec::with_capacity(num_sell_items);
        let mut buy_items = Vec::with_capacity(num_buy_items);

        for _ in 0..num_sell_items {
            let slot = reader.read_u8()?;
            let item = reader.read_item_full()?;
            let price = Money(reader.read_u32()? as i64);

            if let Some(item) = item {
                sell_items.push((slot, item, price));
            }
        }

        for _ in 0..num_buy_items {
            let slot = reader.read_u8()?;
            let item = reader.read_item_full()?;
            let price = Money(reader.read_u32()? as i64);

            if let Some(item) = item {
                buy_items.push((slot, item, price));
            }
        }

        Ok(Self {
            sell_items,
            buy_items,
        })
    }
}

impl From<&PacketServerPersonalStoreItemList> for Packet {
    fn from(packet: &PacketServerPersonalStoreItemList) -> Self {
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
    pub items: Vec<(ItemSlot, Option<Item>)>,
    pub money: Money,
}

impl TryFrom<&Packet> for PacketServerPersonalStoreTransactionUpdateMoneyAndInventory {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PersonalStoreTransactionUpdateMoneyAndInventory as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let money = Money(reader.read_i64()?);
        let num_items = reader.read_u8()? as usize;
        let mut items = Vec::with_capacity(num_items);
        for _ in 0..num_items {
            let item_slot = reader.read_item_slot_u8()?;
            let item = reader.read_item_full()?;
            items.push((item_slot, item));
        }

        Ok(Self { items, money })
    }
}

impl From<&PacketServerPersonalStoreTransactionUpdateMoneyAndInventory> for Packet {
    fn from(packet: &PacketServerPersonalStoreTransactionUpdateMoneyAndInventory) -> Self {
        let mut writer = PacketWriter::new(
            ServerPackets::PersonalStoreTransactionUpdateMoneyAndInventory as u16,
        );
        writer.write_i64(packet.money.0);
        writer.write_u8(packet.items.len() as u8);
        for (slot, item) in packet.items.iter() {
            writer.write_item_slot_u8(*slot);
            writer.write_item_full(item.as_ref());
        }
        writer.into()
    }
}

pub struct PacketServerPersonalStoreTransactionResult {
    pub status: PersonalStoreTransactionStatus,
    pub store_entity_id: ClientEntityId,
    pub update_store_items: Vec<(usize, Option<Item>)>,
}

impl TryFrom<&Packet> for PacketServerPersonalStoreTransactionResult {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PersonalStoreTransactionResult as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let store_entity_id = reader.read_entity_id()?;
        let status = match reader.read_u8()? {
            1 => PersonalStoreTransactionStatus::Cancelled,
            2 => PersonalStoreTransactionStatus::SoldOut,
            3 => PersonalStoreTransactionStatus::NoMoreNeed,
            4 | 5 => PersonalStoreTransactionStatus::BoughtFromStore,
            6 | 7 => PersonalStoreTransactionStatus::SoldToStore,
            _ => return Err(PacketError::InvalidPacket),
        };
        let update_item_count = reader.read_u8()? as usize;
        let mut update_store_items = Vec::with_capacity(update_item_count);
        for _ in 0..update_item_count {
            let slot_index = reader.read_u8()? as usize;
            let item = reader.read_item_full()?;
            update_store_items.push((slot_index, item));
        }

        Ok(Self {
            status,
            store_entity_id,
            update_store_items,
        })
    }
}

impl From<&PacketServerPersonalStoreTransactionResult> for Packet {
    fn from(packet: &PacketServerPersonalStoreTransactionResult) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PersonalStoreTransactionResult as u16);
        writer.write_entity_id(packet.store_entity_id);
        match packet.status {
            PersonalStoreTransactionStatus::Cancelled => writer.write_u8(1),
            PersonalStoreTransactionStatus::SoldOut => writer.write_u8(2),
            PersonalStoreTransactionStatus::NoMoreNeed => writer.write_u8(3),
            PersonalStoreTransactionStatus::BoughtFromStore => writer.write_u8(4),
            PersonalStoreTransactionStatus::SoldToStore => writer.write_u8(6),
        }

        writer.write_u8(packet.update_store_items.len() as u8);
        for (slot_index, item) in packet.update_store_items.iter() {
            writer.write_u8(*slot_index as u8);
            writer.write_item_full(item.as_ref());
        }

        writer.into()
    }
}

pub struct PacketServerUseItem {
    pub entity_id: ClientEntityId,
    pub item: ItemReference,
    pub inventory_slot: Option<ItemSlot>,
}

impl TryFrom<&Packet> for PacketServerUseItem {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UseItem as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let item = ItemReference::new(ItemType::Consumable, reader.read_u16()? as usize);
        let inventory_slot = reader.read_item_slot_u8().ok();

        Ok(Self {
            entity_id,
            item,
            inventory_slot,
        })
    }
}

impl From<&PacketServerUseItem> for Packet {
    fn from(packet: &PacketServerUseItem) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UseItem as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.item.item_number as u16);
        if let Some(inventory_slot) = packet.inventory_slot {
            writer.write_item_slot_u8(inventory_slot);
        }
        writer.into()
    }
}

pub struct PacketServerCastSkillSelf {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub cast_motion_id: Option<MotionId>,
}

impl TryFrom<&Packet> for PacketServerCastSkillSelf {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::CastSkillSelf as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let skill_id = SkillId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let cast_motion_id = reader.read_u8().ok().map(|x| MotionId::new(x as u16));

        Ok(Self {
            entity_id,
            skill_id,
            cast_motion_id,
        })
    }
}

impl From<&PacketServerCastSkillSelf> for Packet {
    fn from(packet: &PacketServerCastSkillSelf) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillSelf as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get());
        if let Some(cast_motion_id) = packet.cast_motion_id {
            writer.write_u8(cast_motion_id.get() as u8);
        }
        writer.into()
    }
}

pub struct PacketServerCastSkillTargetEntity {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_entity_id: ClientEntityId,
    pub target_distance: f32,
    pub target_position: Vec2,
    pub cast_motion_id: Option<MotionId>,
}

impl TryFrom<&Packet> for PacketServerCastSkillTargetEntity {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::CastSkillTargetEntity as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let target_entity_id = reader.read_entity_id()?;
        let skill_id = SkillId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let target_distance = reader.read_u16()? as f32;
        let target_position_x = reader.read_f32()?;
        let target_position_y = reader.read_f32()?;
        let cast_motion_id = reader.read_u8().ok().map(|x| MotionId::new(x as u16));

        Ok(Self {
            entity_id,
            skill_id,
            target_entity_id,
            target_distance,
            target_position: Vec2::new(target_position_x, target_position_y),
            cast_motion_id,
        })
    }
}

impl From<&PacketServerCastSkillTargetEntity> for Packet {
    fn from(packet: &PacketServerCastSkillTargetEntity) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillTargetEntity as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.target_entity_id);
        writer.write_u16(packet.skill_id.get());
        writer.write_u16(packet.target_distance as u16);
        writer.write_f32(packet.target_position.x);
        writer.write_f32(packet.target_position.y);
        if let Some(cast_motion_id) = packet.cast_motion_id {
            writer.write_u8(cast_motion_id.get() as u8);
        }
        writer.into()
    }
}

pub struct PacketServerCastSkillTargetPosition {
    pub entity_id: ClientEntityId,
    pub skill_id: SkillId,
    pub target_position: Vec2,
    pub cast_motion_id: Option<MotionId>,
}

impl TryFrom<&Packet> for PacketServerCastSkillTargetPosition {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::CastSkillTargetPosition as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let skill_id = SkillId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;
        let target_position_x = reader.read_f32()?;
        let target_position_y = reader.read_f32()?;
        let cast_motion_id = reader.read_u8().ok().map(|x| MotionId::new(x as u16));

        Ok(Self {
            entity_id,
            skill_id,
            target_position: Vec2::new(target_position_x, target_position_y),
            cast_motion_id,
        })
    }
}

impl From<&PacketServerCastSkillTargetPosition> for Packet {
    fn from(packet: &PacketServerCastSkillTargetPosition) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CastSkillTargetPosition as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get());
        writer.write_f32(packet.target_position.x);
        writer.write_f32(packet.target_position.y);
        if let Some(cast_motion_id) = packet.cast_motion_id {
            writer.write_u8(cast_motion_id.get() as u8);
        }
        writer.into()
    }
}

pub struct PacketServerStartCastingSkill {
    pub entity_id: ClientEntityId,
}

impl TryFrom<&Packet> for PacketServerStartCastingSkill {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::StartCastingSkill as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        Ok(Self { entity_id })
    }
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
    skill_id: B12,
    effect_success_1: bool,
    effect_success_2: bool,
    caster_intelligence: B10,
}

pub struct PacketServerApplySkillEffect {
    pub entity_id: ClientEntityId,
    pub caster_entity_id: ClientEntityId,
    pub caster_intelligence: i32,
    pub skill_id: SkillId,
    pub effect_success: [bool; 2],
}

impl TryFrom<&Packet> for PacketServerApplySkillEffect {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::ApplySkillEffect as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let caster_entity_id = reader.read_entity_id()?;
        let skill_effect_data =
            SkillEffectData::from_bytes(reader.read_fixed_length_bytes(3)?.try_into().unwrap());

        let skill_id =
            SkillId::new(skill_effect_data.skill_id()).ok_or(PacketError::InvalidPacket)?;
        let caster_intelligence = skill_effect_data.caster_intelligence() as i32;
        let effect_success = [
            skill_effect_data.effect_success_1(),
            skill_effect_data.effect_success_2(),
        ];

        Ok(Self {
            entity_id,
            caster_entity_id,
            caster_intelligence,
            skill_id,
            effect_success,
        })
    }
}

impl From<&PacketServerApplySkillEffect> for Packet {
    fn from(packet: &PacketServerApplySkillEffect) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ApplySkillEffect as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.caster_entity_id);

        let data = SkillEffectData::new()
            .with_skill_id(packet.skill_id.get())
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
    pub is_immediate: bool,
    // TODO: Optional item drop with damage
}

impl TryFrom<&Packet> for PacketServerApplySkillDamage {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::ApplySkillDamage as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let caster_entity_id = reader.read_entity_id()?;
        let skill_effect_data =
            SkillEffectData::from_bytes(reader.read_fixed_length_bytes(3)?.try_into().unwrap());
        let (damage, is_killed, is_immediate) = reader.read_damage_u16()?;

        let skill_id =
            SkillId::new(skill_effect_data.skill_id()).ok_or(PacketError::InvalidPacket)?;
        let caster_intelligence = skill_effect_data.caster_intelligence() as i32;
        let effect_success = [
            skill_effect_data.effect_success_1(),
            skill_effect_data.effect_success_2(),
        ];

        Ok(Self {
            entity_id,
            caster_entity_id,
            caster_intelligence,
            skill_id,
            effect_success,
            damage,
            is_killed,
            is_immediate,
        })
    }
}

impl From<&PacketServerApplySkillDamage> for Packet {
    fn from(packet: &PacketServerApplySkillDamage) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ApplySkillDamage as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_entity_id(packet.caster_entity_id);

        let data = SkillEffectData::new()
            .with_skill_id(packet.skill_id.get())
            .with_effect_success_1(packet.effect_success[0])
            .with_effect_success_2(packet.effect_success[1])
            .with_caster_intelligence(packet.caster_intelligence as u16);
        for b in data.into_bytes().iter() {
            writer.write_u8(*b);
        }

        writer.write_damage_u16(&packet.damage, packet.is_killed, packet.is_immediate);
        writer.into()
    }
}

pub struct PacketServerCancelCastingSkill {
    pub entity_id: ClientEntityId,
    pub reason: CancelCastingSkillReason,
}

impl TryFrom<&Packet> for PacketServerCancelCastingSkill {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::CancelCastingSkill as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let reason = match reader.read_u8()? {
            1 => CancelCastingSkillReason::NeedAbility,
            2 => CancelCastingSkillReason::NeedTarget,
            3 => CancelCastingSkillReason::InvalidTarget,
            _ => return Err(PacketError::InvalidPacket),
        };
        Ok(Self { entity_id, reason })
    }
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

impl TryFrom<&Packet> for PacketServerFinishCastingSkill {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::FinishCastingSkill as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let skill_id = SkillId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;

        Ok(Self {
            entity_id,
            skill_id,
        })
    }
}

impl From<&PacketServerFinishCastingSkill> for Packet {
    fn from(packet: &PacketServerFinishCastingSkill) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::FinishCastingSkill as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u16(packet.skill_id.get());
        writer.into()
    }
}

pub struct PacketServerUpdateSpeed {
    pub entity_id: ClientEntityId,
    pub run_speed: i32,
    pub passive_attack_speed: i32,
}

impl TryFrom<&Packet> for PacketServerUpdateSpeed {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateSpeed as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let run_speed = reader.read_u16()? as i32;
        let passive_attack_speed = reader.read_u16()? as i32;
        let _weight_rate = reader.read_u8()?;

        Ok(Self {
            entity_id,
            run_speed,
            passive_attack_speed,
        })
    }
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

pub struct PacketServerUpdateStatusEffects {
    pub entity_id: ClientEntityId,
    pub status_effects: ActiveStatusEffects,
    pub updated_values: Vec<i32>,
}

impl TryFrom<&Packet> for PacketServerUpdateStatusEffects {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UpdateStatusEffects as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;

        let mut status_effects = ActiveStatusEffects::default();
        reader.read_status_effects_flags_u32(&mut status_effects)?;

        let mut updated_values = Vec::new();
        while let Ok(value) = reader.read_i32() {
            updated_values.push(value);
        }

        Ok(Self {
            entity_id,
            status_effects,
            updated_values,
        })
    }
}

impl From<&PacketServerUpdateStatusEffects> for Packet {
    fn from(packet: &PacketServerUpdateStatusEffects) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UpdateStatusEffects as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_status_effects_flags_u32(&packet.status_effects);

        for value in packet.updated_values.iter() {
            writer.write_i32(*value);
        }

        writer.into()
    }
}

pub struct PacketServerNpcStoreTransactionError {
    pub error: NpcStoreTransactionError,
}

impl TryFrom<&Packet> for PacketServerNpcStoreTransactionError {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::NpcStoreTransactionError as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let error = match reader.read_u8()? {
            1 => NpcStoreTransactionError::PriceDifference,
            2 => NpcStoreTransactionError::NpcNotFound,
            3 => NpcStoreTransactionError::NpcTooFarAway,
            4 => NpcStoreTransactionError::NotEnoughMoney,
            5 => NpcStoreTransactionError::NotSameUnion,
            6 => NpcStoreTransactionError::NotEnoughUnionPoints,
            _ => {
                return Err(PacketError::InvalidPacket);
            }
        };

        Ok(PacketServerNpcStoreTransactionError { error })
    }
}

impl From<&PacketServerNpcStoreTransactionError> for Packet {
    fn from(packet: &PacketServerNpcStoreTransactionError) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::NpcStoreTransactionError as u16);

        let error = match packet.error {
            NpcStoreTransactionError::PriceDifference => 1,
            NpcStoreTransactionError::NpcNotFound => 2,
            NpcStoreTransactionError::NpcTooFarAway => 3,
            NpcStoreTransactionError::NotEnoughMoney => 4,
            NpcStoreTransactionError::NotSameUnion => 5,
            NpcStoreTransactionError::NotEnoughUnionPoints => 6,
        };

        writer.write_u8(error);
        writer.into()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PacketServerMoveToggleType {
    Walk,
    Run,
    Drive,
    Sit,
}

pub struct PacketServerMoveToggle {
    pub entity_id: ClientEntityId,
    pub move_toggle_type: PacketServerMoveToggleType,
    pub run_speed: Option<i32>,
}

impl TryFrom<&Packet> for PacketServerMoveToggle {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::MoveToggle as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let move_toggle_type = match reader.read_u8()? {
            1 => PacketServerMoveToggleType::Sit,
            2 => PacketServerMoveToggleType::Walk,
            3 => PacketServerMoveToggleType::Run,
            4 => PacketServerMoveToggleType::Drive,
            _ => return Err(PacketError::InvalidPacket),
        };
        let run_speed = reader.read_u16().ok().map(|x| x as i32);

        Ok(Self {
            entity_id,
            move_toggle_type,
            run_speed,
        })
    }
}

impl From<&PacketServerMoveToggle> for Packet {
    fn from(packet: &PacketServerMoveToggle) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::MoveToggle as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_u8(match packet.move_toggle_type {
            PacketServerMoveToggleType::Sit => 1,
            PacketServerMoveToggleType::Walk => 2,
            PacketServerMoveToggleType::Run => 3,
            PacketServerMoveToggleType::Drive => 4,
        });
        if let Some(run_speed) = packet.run_speed {
            writer.write_u16(run_speed as u16);
        }
        writer.into()
    }
}

pub struct PacketServerUseEmote {
    pub entity_id: ClientEntityId,
    pub motion_id: MotionId,
    pub is_stop: bool,
}

impl TryFrom<&Packet> for PacketServerUseEmote {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::UseEmote as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let motion_id = MotionId::new(reader.read_u16()?);
        let is_stop = (reader.read_u16()? & 1 << 15) != 0;
        let entity_id = reader.read_entity_id()?;

        Ok(Self {
            entity_id,
            motion_id,
            is_stop,
        })
    }
}

impl From<&PacketServerUseEmote> for Packet {
    fn from(packet: &PacketServerUseEmote) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::UseEmote as u16);
        writer.write_u16(packet.motion_id.get());
        writer.write_u16(if packet.is_stop { 1 << 15 } else { 0 });
        writer.write_entity_id(packet.entity_id);
        writer.into()
    }
}

pub enum PacketServerPartyRequest {
    Create(ClientEntityId),
    Invite(ClientEntityId),
}

impl TryFrom<&Packet> for PacketServerPartyRequest {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyRequest as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let party_request = match reader.read_u8()? {
            0 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyRequest::Create(entity_id)
            }
            1 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyRequest::Invite(entity_id)
            }
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(party_request)
    }
}

impl From<&PacketServerPartyRequest> for Packet {
    fn from(packet: &PacketServerPartyRequest) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyRequest as u16);
        match *packet {
            PacketServerPartyRequest::Create(entity_id) => {
                writer.write_u8(0);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketServerPartyRequest::Invite(entity_id) => {
                writer.write_u8(1);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
        }
        writer.into()
    }
}

pub enum PacketServerPartyReply {
    AcceptCreate(ClientEntityId),
    AcceptInvite(ClientEntityId),
    RejectInvite(PartyRejectInviteReason, ClientEntityId),
    Delete,
    ChangeOwner(ClientEntityId),
    MemberKicked(u32),
    MemberDisconnect(u32),
}

impl TryFrom<&Packet> for PacketServerPartyReply {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyReply as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let party_reply = match reader.read_u8()? {
            1 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyReply::RejectInvite(PartyRejectInviteReason::Busy, entity_id)
            }
            2 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyReply::AcceptCreate(entity_id)
            }
            3 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyReply::AcceptInvite(entity_id)
            }
            4 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyReply::RejectInvite(PartyRejectInviteReason::Reject, entity_id)
            }
            5 => PacketServerPartyReply::Delete,
            8 => {
                let entity_id = reader.read_entity_id()?;
                PacketServerPartyReply::ChangeOwner(entity_id)
            }
            0x80 => {
                let character_id = reader.read_u32()?;
                PacketServerPartyReply::MemberKicked(character_id)
            }
            0x81 => {
                let character_id = reader.read_u32()?;
                PacketServerPartyReply::MemberDisconnect(character_id)
            }
            _ => return Err(PacketError::InvalidPacket),
        };

        Ok(party_reply)
    }
}

impl From<&PacketServerPartyReply> for Packet {
    fn from(packet: &PacketServerPartyReply) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyReply as u16);
        match *packet {
            PacketServerPartyReply::AcceptCreate(entity_id) => {
                writer.write_u8(2);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketServerPartyReply::AcceptInvite(entity_id) => {
                writer.write_u8(3);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketServerPartyReply::RejectInvite(reason, entity_id) => {
                match reason {
                    PartyRejectInviteReason::Busy => writer.write_u8(1),
                    PartyRejectInviteReason::Reject => writer.write_u8(4),
                }
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketServerPartyReply::Delete => {
                writer.write_u8(5);
                writer.write_u32(0);
            }
            PacketServerPartyReply::ChangeOwner(entity_id) => {
                writer.write_u8(8);
                writer.write_entity_id(entity_id);
                writer.write_u16(0);
            }
            PacketServerPartyReply::MemberKicked(character_id) => {
                writer.write_u8(0x80);
                writer.write_u32(character_id);
            }
            PacketServerPartyReply::MemberDisconnect(character_id) => {
                writer.write_u8(0x81);
                writer.write_u32(character_id);
            }
        }
        writer.into()
    }
}

pub enum PacketServerPartyMembers {
    List(PartyMemberList),
    Leave(PartyMemberLeave),
}

impl TryFrom<&Packet> for PacketServerPartyMembers {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyMembers as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let (item_sharing, xp_sharing) = reader.read_party_rules()?;
        let num_members = reader.read_u8()?;

        if num_members == 255 {
            let leaver_character_id = reader.read_u32()?;
            let owner_character_id = reader.read_u32()?;

            Ok(PacketServerPartyMembers::Leave(PartyMemberLeave {
                leaver_character_id,
                owner_character_id,
            }))
        } else {
            let mut members = Vec::with_capacity(num_members as usize);
            for _ in 0..num_members {
                members.push(reader.read_party_member_info()?);
            }

            Ok(PacketServerPartyMembers::List(PartyMemberList {
                item_sharing,
                xp_sharing,
                owner_character_id: members[0].get_character_id(),
                members,
            }))
        }
    }
}

impl From<&PacketServerPartyMembers> for Packet {
    fn from(packet: &PacketServerPartyMembers) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyMembers as u16);
        match *packet {
            PacketServerPartyMembers::List(PartyMemberList {
                item_sharing,
                xp_sharing,
                owner_character_id,
                ref members,
            }) => {
                writer.write_party_rules(&item_sharing, &xp_sharing);
                writer.write_u8(members.len() as u8);

                // Owner is the first member in packet
                for party_member in members.iter() {
                    if party_member.get_character_id() == owner_character_id {
                        writer.write_party_member_info(party_member);
                    }
                }

                for party_member in members.iter() {
                    if party_member.get_character_id() != owner_character_id {
                        writer.write_party_member_info(party_member);
                    }
                }
            }
            PacketServerPartyMembers::Leave(PartyMemberLeave {
                owner_character_id,
                leaver_character_id,
            }) => {
                writer.write_u8(0);
                writer.write_u8(255); // -1
                writer.write_u32(leaver_character_id);
                writer.write_u32(owner_character_id);
            }
        }
        writer.into()
    }
}

pub struct PacketServerPartyMemberUpdateInfo {
    pub member_info: PartyMemberInfoOnline,
}

impl TryFrom<&Packet> for PacketServerPartyMemberUpdateInfo {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyMemberUpdateInfo as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let member_info = reader.read_party_member_info_online()?;
        Ok(PacketServerPartyMemberUpdateInfo { member_info })
    }
}

impl From<&PacketServerPartyMemberUpdateInfo> for Packet {
    fn from(packet: &PacketServerPartyMemberUpdateInfo) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyMemberUpdateInfo as u16);
        writer.write_party_member_info_online(&packet.member_info);
        writer.into()
    }
}

pub struct PacketServerPartyMemberRewardItem {
    pub entity_id: ClientEntityId,
    pub item: Item,
}

impl TryFrom<&Packet> for PacketServerPartyMemberRewardItem {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyMemberRewardItem as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let entity_id = reader.read_entity_id()?;
        let item = reader.read_item_full()?.ok_or(PacketError::InvalidPacket)?;
        Ok(Self { entity_id, item })
    }
}

impl From<&PacketServerPartyMemberRewardItem> for Packet {
    fn from(packet: &PacketServerPartyMemberRewardItem) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyMemberRewardItem as u16);
        writer.write_entity_id(packet.entity_id);
        writer.write_item_full(Some(&packet.item));
        writer.into()
    }
}

pub struct PacketServerChangeNpcId {
    pub client_entity_id: ClientEntityId,
    pub npc_id: NpcId,
}

impl TryFrom<&Packet> for PacketServerChangeNpcId {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::ChangeNpcId as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let client_entity_id = reader.read_entity_id()?;
        let npc_id = NpcId::new(reader.read_u16()?).ok_or(PacketError::InvalidPacket)?;

        Ok(Self {
            client_entity_id,
            npc_id,
        })
    }
}

impl From<&PacketServerChangeNpcId> for Packet {
    fn from(packet: &PacketServerChangeNpcId) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ChangeNpcId as u16);
        writer.write_entity_id(packet.client_entity_id);
        writer.write_u16(packet.npc_id.get());
        writer.into()
    }
}

pub struct PacketServerPartyUpdateRules {
    pub item_sharing: PartyItemSharing,
    pub xp_sharing: PartyXpSharing,
}

impl TryFrom<&Packet> for PacketServerPartyUpdateRules {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::PartyUpdateRules as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let (item_sharing, xp_sharing) = reader.read_party_rules()?;

        Ok(Self {
            item_sharing,
            xp_sharing,
        })
    }
}

impl From<&PacketServerPartyUpdateRules> for Packet {
    fn from(packet: &PacketServerPartyUpdateRules) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::PartyUpdateRules as u16);
        writer.write_party_rules(&packet.item_sharing, &packet.xp_sharing);
        writer.into()
    }
}

pub struct PacketServerAdjustPosition {
    pub client_entity_id: ClientEntityId,
    pub position: Vec3,
}

impl TryFrom<&Packet> for PacketServerAdjustPosition {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::AdjustPosition as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let client_entity_id = reader.read_entity_id()?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_i16()? as f32;
        Ok(Self {
            client_entity_id,
            position: Vec3::new(x, y, z),
        })
    }
}

impl From<&PacketServerAdjustPosition> for Packet {
    fn from(packet: &PacketServerAdjustPosition) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::AdjustPosition as u16);
        writer.write_entity_id(packet.client_entity_id);
        writer.write_f32(packet.position.x);
        writer.write_f32(packet.position.y);
        writer.write_i16(packet.position.z as i16);
        writer.into()
    }
}

pub enum PacketServerCraftItem {
    InsertGemFailed {
        error: CraftInsertGemError,
    },
    InsertGemSuccess {
        items: Vec<(ItemSlot, Option<Item>)>,
    },
}

impl TryFrom<&Packet> for PacketServerCraftItem {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::CraftItem as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let craft_type = reader.read_u8()?;
        match craft_type {
            1 => {
                let num_items = reader.read_u8()? as usize;
                let mut items = Vec::with_capacity(num_items);
                for _ in 0..num_items {
                    let item_slot = reader.read_item_slot_u8()?;
                    let item = reader.read_item_full()?;
                    items.push((item_slot, item));
                }
                Ok(Self::InsertGemSuccess { items })
            }
            2 => Ok(Self::InsertGemFailed {
                error: CraftInsertGemError::NoSocket,
            }),
            3 => Ok(Self::InsertGemFailed {
                error: CraftInsertGemError::SocketFull,
            }),
            // TODO: 4, 5, 6, 7, 16, 17, 18
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

impl From<&PacketServerCraftItem> for Packet {
    fn from(packet: &PacketServerCraftItem) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::CraftItem as u16);

        match packet {
            PacketServerCraftItem::InsertGemSuccess { items } => {
                writer.write_u8(1);
                writer.write_u8(items.len() as u8);

                for (slot, item) in items.iter() {
                    writer.write_item_slot_u8(*slot);
                    writer.write_item_full(item.as_ref());
                }
            }
            PacketServerCraftItem::InsertGemFailed {
                error: CraftInsertGemError::NoSocket,
            } => {
                writer.write_u8(2);
            }
            PacketServerCraftItem::InsertGemFailed {
                error: CraftInsertGemError::SocketFull,
            } => {
                writer.write_u8(3);
            }
        }

        writer.into()
    }
}

pub enum PacketServerBankOpen {
    Open,
    SetItems { items: Vec<(u8, Option<Item>)> },
    UpdateItems { items: Vec<(u8, Option<Item>)> },
}

impl TryFrom<&Packet> for PacketServerBankOpen {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::BankOpen as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let command = reader.read_u8()?;
        match command {
            0 => {
                let num_items = reader.read_u8()? as usize;
                let mut items = Vec::with_capacity(num_items);
                for _ in 0..num_items {
                    let storage_slot = reader.read_u8()?;
                    let item = reader.read_item_full()?;
                    items.push((storage_slot, item));
                }

                Ok(Self::SetItems { items })
            }
            1 => Ok(Self::Open),
            2 => {
                let num_items = reader.read_u8()? as usize;
                let mut items = Vec::with_capacity(num_items);
                for _ in 0..num_items {
                    let storage_slot = reader.read_u8()?;
                    let item = reader.read_item_full()?;
                    items.push((storage_slot, item));
                }

                Ok(Self::UpdateItems { items })
            }
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

impl From<&PacketServerBankOpen> for Packet {
    fn from(packet: &PacketServerBankOpen) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::BankOpen as u16);
        match packet {
            PacketServerBankOpen::SetItems { items } => {
                writer.write_u8(0);
                writer.write_u8(items.len() as u8);
                for (storage_slot, item) in items.iter() {
                    writer.write_u8(*storage_slot);
                    writer.write_item_full(item.as_ref());
                }
            }
            PacketServerBankOpen::Open => {
                writer.write_u8(1);
                writer.write_u8(0);
            }
            PacketServerBankOpen::UpdateItems { items } => {
                writer.write_u8(2);
                writer.write_u8(items.len() as u8);
                for (storage_slot, item) in items.iter() {
                    writer.write_u8(*storage_slot);
                    writer.write_item_full(item.as_ref());
                }
            }
        }
        writer.into()
    }
}

pub struct PacketServerBankTransaction {
    pub inventory_item_slot: ItemSlot,
    pub inventory_item: Option<Item>,
    pub inventory_money: Option<Money>,
    pub bank_slot: usize,
    pub bank_item: Option<Item>,
}

impl TryFrom<&Packet> for PacketServerBankTransaction {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::BankTransaction as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let inventory_item_slot = reader.read_item_slot_u16()?;
        let bank_slot = reader.read_u16()? as usize;
        let inventory_item = reader.read_item_full()?;
        let bank_item = reader.read_item_full()?;
        let inventory_money = reader.read_i64().ok().map(Money);

        Ok(Self {
            inventory_item_slot,
            inventory_item,
            inventory_money,
            bank_slot,
            bank_item,
        })
    }
}

impl From<&PacketServerBankTransaction> for Packet {
    fn from(packet: &PacketServerBankTransaction) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::BankTransaction as u16);
        writer.write_item_slot_u16(packet.inventory_item_slot);
        writer.write_u16(packet.bank_slot as u16);
        writer.write_item_full(packet.inventory_item.as_ref());
        writer.write_item_full(packet.bank_item.as_ref());
        if let Some(inventory_money) = packet.inventory_money.as_ref() {
            writer.write_i64(inventory_money.0);
        }
        writer.into()
    }
}

pub struct PacketServerRepairedItemUsingNpc {
    pub item_slot: ItemSlot,
    pub item: Item,
    pub updated_money: Money,
}

impl TryFrom<&Packet> for PacketServerRepairedItemUsingNpc {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::RepairedItemUsingNpc as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let updated_money = Money(reader.read_i64()?);
        let num_items = reader.read_u8()? as usize;
        if num_items != 1 {
            return Err(PacketError::InvalidPacket);
        }
        let item_slot = reader.read_item_slot_u8()?;
        let item = reader.read_item_full()?.ok_or(PacketError::InvalidPacket)?;

        Ok(Self {
            item_slot,
            item,
            updated_money,
        })
    }
}

impl From<&PacketServerRepairedItemUsingNpc> for Packet {
    fn from(packet: &PacketServerRepairedItemUsingNpc) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::RepairedItemUsingNpc as u16);
        writer.write_i64(packet.updated_money.0);
        writer.write_u8(1);
        writer.write_item_slot_u8(packet.item_slot);
        writer.write_item_full(Some(&packet.item));
        writer.into()
    }
}

#[derive(Debug)]
pub enum PacketServerClanCommand {
    ClanInfo {
        id: ClanUniqueId,
        name: String,
        description: String,
        mark: ClanMark,
        level: ClanLevel,
        points: ClanPoints,
        money: Money,
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

impl TryFrom<&Packet> for PacketServerClanCommand {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, PacketError> {
        if packet.command != ServerPackets::ClanCommand as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        match reader.read_u8()? {
            0x33 => {
                let id = ClanUniqueId::new(reader.read_u32()?).ok_or(PacketError::InvalidPacket)?;
                let mark = reader.read_clan_mark_u32()?;
                let level =
                    ClanLevel::new(reader.read_u8()? as u32).ok_or(PacketError::InvalidPacket)?;
                let position = reader.read_clan_member_position_u8()?;

                let points = ClanPoints(reader.read_u32()? as u64);
                let _storage_rate = reader.read_u32()?;
                let money = Money(reader.read_i64()?);
                let _member_count = reader.read_u16()?;
                let contribution = ClanPoints(reader.read_u32()? as u64);

                let mut skills = Vec::new();
                for _ in 0..20 {
                    let skill_id = reader.read_u16()?;
                    let _expire_secs = reader.read_u32()?;
                    if let Some(skill_id) = SkillId::new(skill_id) {
                        skills.push(skill_id);
                    }
                }

                let name = reader.read_null_terminated_utf8()?.to_string();
                let description = reader
                    .read_null_terminated_utf8()
                    .unwrap_or_default()
                    .to_string();

                Ok(Self::ClanInfo {
                    id,
                    name,
                    description,
                    mark,
                    level,
                    points,
                    money,
                    position,
                    contribution,
                    skills,
                })
            }
            0x35 => {
                let client_entity_id = reader.read_entity_id()?;
                let id = ClanUniqueId::new(reader.read_u32()?).ok_or(PacketError::InvalidPacket)?;
                let mark = reader.read_clan_mark_u32()?;
                let level =
                    ClanLevel::new(reader.read_u8()? as u32).ok_or(PacketError::InvalidPacket)?;
                let position = reader.read_clan_member_position_u8()?;
                let name = reader.read_null_terminated_utf8()?.to_string();

                Ok(Self::CharacterUpdateClan {
                    client_entity_id,
                    id,
                    name,
                    mark,
                    level,
                    position,
                })
            }
            0x41 => Ok(Self::ClanCreateError {
                error: ClanCreateError::Failed,
            }),
            0x42 => Ok(Self::ClanCreateError {
                error: ClanCreateError::NameExists,
            }),
            0x43 => Ok(Self::ClanCreateError {
                error: ClanCreateError::NoPermission,
            }),
            0x44 => Ok(Self::ClanCreateError {
                error: ClanCreateError::UnmetCondition,
            }),
            0x71 => {
                let id = ClanUniqueId::new(reader.read_u32()?).ok_or(PacketError::InvalidPacket)?;
                let mark = reader.read_clan_mark_u32()?;
                let level =
                    ClanLevel::new(reader.read_u8()? as u32).ok_or(PacketError::InvalidPacket)?;
                let _position = reader.read_u8()?;

                let points = ClanPoints(reader.read_u32()? as u64);
                let _storage_rate = reader.read_u32()?;
                let money = Money(reader.read_i64()?);
                let _member_count = reader.read_u16()?;
                let _contribution = reader.read_u32()?;

                let mut skills = Vec::new();
                for _ in 0..20 {
                    let skill_id = reader.read_u16()?;
                    let _expire_secs = reader.read_u32()?;
                    if let Some(skill_id) = SkillId::new(skill_id) {
                        skills.push(skill_id);
                    }
                }

                Ok(Self::ClanUpdateInfo {
                    id,
                    mark,
                    level,
                    points,
                    money,
                    skills,
                })
            }
            0x72 => {
                let try_read_clan_member =
                    |reader: &mut PacketReader| -> Result<ClanMemberInfo, PacketError> {
                        let position = reader.read_clan_member_position_u8()?;
                        let channel_id = NonZeroUsize::new(reader.read_u8()? as usize);
                        let contribution = ClanPoints(reader.read_u32()? as u64);
                        let level = Level::new(reader.read_u16()? as u32);
                        let job = reader.read_u16()?;
                        let name = reader.read_null_terminated_utf8()?.to_string();

                        Ok(ClanMemberInfo {
                            name,
                            position,
                            contribution,
                            channel_id,
                            level,
                            job,
                        })
                    };

                let mut members = Vec::new();
                while let Ok(member) = try_read_clan_member(&mut reader) {
                    members.push(member);
                }

                Ok(Self::ClanMemberList { members })
            }
            0x73 | 0x74 => {
                let _position = reader.read_u8()?;
                let channel_id = NonZeroUsize::new(reader.read_u8()? as usize);
                let _contribution = reader.read_u32()?;
                let _level = reader.read_u16()?;
                let _job = reader.read_u16()?;
                let name = reader.read_null_terminated_utf8()?.to_string();

                if let Some(channel_id) = channel_id {
                    Ok(Self::ClanMemberConnected { name, channel_id })
                } else {
                    Ok(Self::ClanMemberDisconnected { name })
                }
            }
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

impl From<&PacketServerClanCommand> for Packet {
    fn from(packet: &PacketServerClanCommand) -> Self {
        let mut writer = PacketWriter::new(ServerPackets::ClanCommand as u16);

        match packet {
            PacketServerClanCommand::ClanInfo {
                id,
                mark,
                level,
                position,
                contribution,
                points,
                money,
                name,
                description,
                skills,
            } => {
                writer.write_u8(0x33);

                // tag_CLAN_ID
                writer.write_u32(id.get());
                writer.write_clan_mark_u32(mark);
                writer.write_u8(level.get() as u8);
                writer.write_clan_member_position_u8(position);

                // tag_MY_CLAN
                writer.write_u32(points.0 as u32);
                writer.write_u32(0); // unused: clan storage rate
                writer.write_i64(money.0);
                writer.write_u16(0); // unused: member count
                writer.write_u32(contribution.0 as u32);

                // 20 clan skills
                for i in 0..20 {
                    if let Some(skill_id) = skills.get(i) {
                        writer.write_u16(skill_id.get());
                        writer.write_u32(0); // TODO: Expire secs
                    } else {
                        writer.write_u16(0);
                        writer.write_u32(0);
                    }
                }

                writer.write_null_terminated_utf8(name);
                if !description.is_empty() {
                    writer.write_null_terminated_utf8(description);
                }
            }
            PacketServerClanCommand::CharacterUpdateClan {
                client_entity_id,
                id,
                name,
                mark,
                level,
                position,
            } => {
                writer.write_u8(0x35);

                writer.write_entity_id(*client_entity_id);

                writer.write_u32(id.get());
                writer.write_clan_mark_u32(mark);
                writer.write_u8(level.get() as u8);
                writer.write_clan_member_position_u8(position);

                writer.write_null_terminated_utf8(name);
            }
            PacketServerClanCommand::ClanUpdateInfo {
                id,
                mark,
                level,
                points,
                money,
                skills,
            } => {
                writer.write_u8(0x71);

                // tag_CLAN_ID
                writer.write_u32(id.get());
                writer.write_clan_mark_u32(mark);
                writer.write_u8(level.get() as u8);
                writer.write_u8(0); // unused: position

                // tag_MY_CLAN
                writer.write_u32(points.0 as u32);
                writer.write_u32(0); // unused: clan storage rate
                writer.write_i64(money.0);
                writer.write_u16(0); // unused: member count
                writer.write_u32(0); // unused: contribution

                // 20 clan skills
                for i in 0..20 {
                    if let Some(skill_id) = skills.get(i) {
                        writer.write_u16(skill_id.get());
                        writer.write_u32(0); // TODO: Expire secs
                    } else {
                        writer.write_u16(0);
                        writer.write_u32(0);
                    }
                }
            }
            PacketServerClanCommand::ClanMemberList { members } => {
                writer.write_u8(0x72);

                for member in members.iter() {
                    writer.write_clan_member_position_u8(&member.position);
                    writer.write_u8(member.channel_id.map_or(0, |value| value.get()) as u8);
                    writer.write_u32(member.contribution.0 as u32);
                    writer.write_u16(member.level.level as u16);
                    writer.write_u16(member.job);
                    writer.write_null_terminated_utf8(&member.name);
                }
            }
            PacketServerClanCommand::ClanMemberConnected { name, channel_id } => {
                writer.write_u8(0x73);

                writer.write_u8(0); // unused: position
                writer.write_u8(channel_id.get() as u8);
                writer.write_u32(0); // unused: contribution
                writer.write_u16(0); // unused: level
                writer.write_u16(0); // unused: job
                writer.write_null_terminated_utf8(name);
            }
            PacketServerClanCommand::ClanMemberDisconnected { name } => {
                writer.write_u8(0x74);

                writer.write_u8(0); // unused: position
                writer.write_u8(0); // channel id 0 required for logged out
                writer.write_u32(0); // unused: contribution
                writer.write_u16(0); // unused: level
                writer.write_u16(0); // unused: job
                writer.write_null_terminated_utf8(name);
            }
            PacketServerClanCommand::ClanCreateError { error } => match error {
                ClanCreateError::Failed => writer.write_u8(0x41),
                ClanCreateError::NameExists => writer.write_u8(0x42),
                ClanCreateError::NoPermission => writer.write_u8(0x43),
                ClanCreateError::UnmetCondition => writer.write_u8(0x44),
            },
        }

        writer.into()
    }
}
