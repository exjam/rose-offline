use std::{num::NonZeroUsize, time::Duration};

use bevy::math::Vec3;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use rose_data::{
    AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemReference, ItemType, NpcId, SkillId,
    SkillPageType, StackableItem, VehiclePartIndex, WorldTicks, ZoneId,
};
use rose_game_common::{
    components::{
        BasicStats, CharacterInfo, Equipment, ExperiencePoints, HealthPoints, Hotbar,
        InventoryPageType, ItemSlot, Level, ManaPoints, Money, MoveMode, Npc, SkillList,
        SkillPoints, SkillSlot, Stamina, StatPoints, Team, UnionMembership, INVENTORY_PAGE_SIZE,
        SKILL_PAGE_SIZE,
    },
    messages::{
        server::{ActiveStatusEffects, CommandState, UpdateSkillData},
        ClientEntityId,
    },
};
use rose_network_common::{Packet, PacketError, PacketReader, PacketWriter};

use crate::common_packets::{
    PacketReadCharacterGender, PacketReadCommandState, PacketReadEntityId, PacketReadItems,
    PacketReadMoveMode, PacketWriteEntityId, PacketWriteMoveMode,
};

#[derive(FromPrimitive)]
pub enum ServerPackets {
    ConnectReply = 0x70c,
    SelectCharacter = 0x715,
    CharacterInventory = 0x716,
    UpdateSkillList = 0x71a,
    // QuestList = 0x71b,
    // UpdateMoney = 0x71d,
    // RewardMoney = 0x71e,
    // RewardItems = 0x71f,
    // UpdateAbilityValueRewardAdd = 0x720,
    // UpdateAbilityValueRewardSet = 0x721,
    // QuestItemList = 0x723,
    // WishList = 0x724,
    // PreloadCharacter = 0x729,
    JoinZone = 0x753,
    SpawnEntityNpc = 0x791,
    SpawnEntityMonster = 0x792,
    // SpawnEntityCharacter = 0x793,
    RemoveEntities = 0x794,
    // StopMoveEntity = 0x796,
    MoveEntityWithMoveMode = 0x797,
    MoveEntity = 0x79a,
    Teleport = 0x7a8,
    UpdateSpeed = 0x7b8,
    // AddEventObject = 0x7d6,
    // UpdateTeam = 0x7f6,
    // PremiumInfo = 0x817,
    // ServerTime = 0x826,
    // QuestCompletionData = 0x855,
    // SkillStatus = 0x862,
    // QuestEmoticon = 0x867,
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

        let zone_id = ZoneId::new(reader.read_u32()? as u16).ok_or(PacketError::InvalidPacket)?;
        let position_x = reader.read_f32()?;
        let position_y = reader.read_f32()?;
        let revive_zone_id =
            ZoneId::new(reader.read_u32()? as u16).ok_or(PacketError::InvalidPacket)?;

        let _face = reader
            .read_equipment_item_part(ItemType::Face)?
            .map_or(0, |i| i.item.item_number);
        let _head = reader
            .read_equipment_item_part(ItemType::Head)?
            .map_or(0, |i| i.item.item_number);
        let equipment = reader.read_equipment_visible_part()?;

        // tagBasicInfo
        let _hair_color = reader.read_u8()?;
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
        let health_points = HealthPoints::new(reader.read_i32()?);
        let mana_points = ManaPoints::new(reader.read_i32()?);
        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let level = Level::new(reader.read_u16()? as u32);
        let stat_points = StatPoints::new(reader.read_u16()? as u32);
        let skill_points = SkillPoints::new(reader.read_u16()? as u32);
        let _penalty_xp = reader.read_u32()?;
        let stamina = Stamina::new(reader.read_u16()? as u32);
        let _pat_hp = reader.read_u32()?;
        let _pat_cooldown = reader.read_u32()?;

        // TODO: Currency
        for _ in 0..10 {
            let _currency_n = reader.read_u32()?;
        }

        // TODO: tagMaintainSTATUS
        for _ in 0..40 {
            let _seconds_remaining = reader.read_u32()?;
            let _buff_id = reader.read_u16()?;
            let _reserved = reader.read_u16()?;
        }

        // TODO: CHotIcons
        let hotbar = Hotbar::default();
        for _ in 0..48 {
            let _hotbar_n = reader.read_u16()?;
        }

        let unique_id = reader.read_u32()?;

        //TODO: Skill cooldowns
        for _ in 0..20 {
            let _cooldown_n = reader.read_u32()?;
        }

        let name = reader.read_null_terminated_utf8()?.to_string();

        Ok(PacketServerSelectCharacter {
            character_info: CharacterInfo {
                name,
                gender,
                race: 0,
                birth_stone: 0,
                job,
                face,
                hair,
                rank,
                fame,
                fame_b: 0,
                fame_g: 0,
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
            skill_list: SkillList::default(),
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
        let entity_id = reader.read_entity_id()?;
        let health_points = HealthPoints::new(reader.read_u16()? as i32);
        let mana_points = ManaPoints::new(reader.read_u16()? as i32);

        let experience_points = ExperiencePoints::new(reader.read_u32()? as u64);
        let _penalty_xp = reader.read_u32();

        // tagVAR_GLOBAL
        let _game_arena_energy_reduction_rate = reader.read_u32()?;
        let _refine_event_bonus = reader.read_u16()?;
        let _empowerment_event_bonus = reader.read_u16()?;
        let _reinforcement_event_bonus = reader.read_u16()?;
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
        let _quest_emoticon = reader.read_u16()?;

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

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum SkillListUpdateType {
    Initial = 1,
    Update = 2,
}

pub struct PacketServerUpdateSkillList {
    pub update_type: SkillListUpdateType,
    pub skill_data: Vec<UpdateSkillData>,
}

impl TryFrom<&Packet> for PacketServerUpdateSkillList {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::UpdateSkillList as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let update_type =
            FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let skill_count = reader.read_u16()? as usize;
        let mut skill_data = Vec::with_capacity(skill_count);
        for _ in 0..skill_count {
            let skill_slot_index = reader.read_u16()? as usize;
            let skill_id = SkillId::new(reader.read_u16()?);
            let expire_time = match reader.read_u32()? {
                0 => None,
                seconds => Some(Duration::from_secs(seconds as u64)),
            };

            // TODO: We need to find a way to support both irose + evo skill layout in SkillList / SkillSlot
            if skill_slot_index < SKILL_PAGE_SIZE {
                skill_data.push(UpdateSkillData {
                    skill_slot: SkillSlot(SkillPageType::Basic, skill_slot_index),
                    skill_id,
                    expire_time,
                });
            }
        }

        Ok(Self {
            update_type,
            skill_data,
        })
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

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum CharacterInventoryUpdateType {
    Initial = 1,
    Update = 2,
}

pub struct PacketServerCharacterInventory {
    pub update_type: CharacterInventoryUpdateType,
    pub items: Vec<(ItemSlot, Option<Item>)>,
    pub money: Money,
}

impl TryFrom<&Packet> for PacketServerCharacterInventory {
    type Error = PacketError;

    fn try_from(packet: &Packet) -> Result<Self, Self::Error> {
        if packet.command != ServerPackets::CharacterInventory as u16 {
            return Err(PacketError::InvalidPacket);
        }

        let mut reader = PacketReader::from(packet);
        let update_type =
            FromPrimitive::from_u8(reader.read_u8()?).ok_or(PacketError::InvalidPacket)?;
        let money = Money(reader.read_i64()?);
        let item_count = reader.read_u32()? as usize;
        let mut items = Vec::with_capacity(item_count);
        for _ in 0..item_count {
            let item_type = reader.read_u16()?;
            let item_number = reader.read_u32()? as usize;
            let _character_unique_id = reader.read_u32()?;
            let _account_unique_id = reader.read_u32()?;
            let _item_color = reader.read_u32()?;

            let _item_key = reader.read_u64()?;
            let _is_crafted = reader.read_u8()?;
            let _gem_option1 = reader.read_u16()?;
            let _gem_option2 = reader.read_u16()?;
            let _gem_option3 = reader.read_u16()?;
            let durability = reader.read_u16()?;
            let _item_life = reader.read_u16()?;
            let _socket_count = reader.read_u8()?;
            let _is_appraised = reader.read_u8()?;
            let _grade = reader.read_u16()?;
            let quantity = reader.read_u16()?;
            let location_id = reader.read_u8()?;
            let slot_index = reader.read_u32()? as usize;
            let _pickup_date_year = reader.read_u16()?;
            let _pickup_date_month = reader.read_u16()?;
            let _pickup_date_day = reader.read_u16()?;
            let _pickup_date_hour = reader.read_u16()?;
            let _pickup_date_minute = reader.read_u16()?;
            let _pickup_date_second = reader.read_u16()?;
            let _pickup_date_milliseconds = reader.read_u16()?;
            let _time_remaining = reader.read_u32()?;
            let _move_limits = reader.read_u16()?;
            let _bind_on_acquire = reader.read_u8()?;
            let _bind_on_equip_use = reader.read_u8()?;
            let _money = reader.read_u32()?;

            let item_type = match item_type {
                1 => ItemType::Face,
                2 => ItemType::Head,
                3 => ItemType::Body,
                4 => ItemType::Hands,
                5 => ItemType::Feet,
                6 => ItemType::Back,
                7 => ItemType::Jewellery,
                8 => ItemType::Weapon,
                9 => ItemType::SubWeapon,
                10 => ItemType::Consumable,
                11 => ItemType::Gem,
                12 => ItemType::Material,
                13 => ItemType::Quest,
                14 => ItemType::Vehicle,
                _ => continue,
            };

            let item = if item_type.is_stackable_item() {
                let item =
                    StackableItem::new(ItemReference::new(item_type, item_number), quantity as u32)
                        .unwrap();
                Item::Stackable(item)
            } else {
                let item = EquipmentItem::new(
                    ItemReference::new(item_type, item_number),
                    durability as u8,
                )
                .unwrap();
                Item::Equipment(item)
            };

            match location_id {
                1 => {
                    let page = match slot_index / INVENTORY_PAGE_SIZE {
                        0 => InventoryPageType::Equipment,
                        1 => InventoryPageType::Consumables,
                        2 => InventoryPageType::Materials,
                        3 => InventoryPageType::Vehicles,
                        _ => continue,
                    };

                    items.push((
                        ItemSlot::Inventory(page, slot_index % INVENTORY_PAGE_SIZE),
                        Some(item),
                    ));
                }
                2 => {
                    let equipment_index = match slot_index {
                        1 => EquipmentIndex::Face,
                        2 => EquipmentIndex::Head,
                        3 => EquipmentIndex::Body,
                        4 => EquipmentIndex::Back,
                        5 => EquipmentIndex::Hands,
                        6 => EquipmentIndex::Feet,
                        7 => EquipmentIndex::Weapon,
                        8 => EquipmentIndex::SubWeapon,
                        9 => EquipmentIndex::Necklace,
                        10 => EquipmentIndex::Ring,
                        11 => EquipmentIndex::Earring,
                        _ => continue,
                    };

                    items.push((ItemSlot::Equipment(equipment_index), Some(item)));
                }
                3 => {
                    let ammo_index = match slot_index {
                        0 => AmmoIndex::Arrow,
                        1 => AmmoIndex::Bullet,
                        2 => AmmoIndex::Throw,
                        _ => continue,
                    };

                    items.push((ItemSlot::Ammo(ammo_index), Some(item)));
                }
                5 => {
                    let vehicle_part_index = match slot_index {
                        0 => VehiclePartIndex::Body,
                        1 => VehiclePartIndex::Engine,
                        2 => VehiclePartIndex::Leg,
                        3 => VehiclePartIndex::Ability,
                        4 => VehiclePartIndex::Arms,
                        _ => continue,
                    };

                    items.push((ItemSlot::Vehicle(vehicle_part_index), Some(item)));
                }
                _ => {} // TODO
            }
        }

        Ok(PacketServerCharacterInventory {
            update_type,
            money,
            items,
        })
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
        let _path_destination_x = reader.read_f32()?;
        let _path_destination_y = reader.read_f32()?;
        let _waypoint_destination_x = reader.read_f32()?;
        let _waypoint_destination_y = reader.read_f32()?;
        let _is_path_end = reader.read_u8()?;
        let _skill_index = reader.read_u16()?;
        let command = reader.read_command_state()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let _passenger_entity_id = reader.read_option_entity_id()?;
        let move_mode = reader.read_move_mode_u8()?;
        let health = HealthPoints::new(reader.read_i32()?);
        let _max_health = reader.read_i32()?;
        let _mana = reader.read_i32()?;
        let _max_mana = reader.read_i32()?;
        let _attack_range_rate = reader.read_i32()?;
        let _attack_range_value = reader.read_i32()?;
        let _skill_range_rate = reader.read_i32()?;
        let _skill_range_value = reader.read_i32()?;
        let team = Team::new(reader.read_u32()?);
        let _sub_flags = reader.read_u64()?;
        let _status_count = reader.read_u16()?;

        let npc_id = reader.read_i16()?;
        let _npc_is_visible = npc_id > 0;
        let npc_id = NpcId::new(npc_id.unsigned_abs()).ok_or(PacketError::InvalidPacket)?;
        let quest_index = reader.read_u16()?;
        let _skill_motion = reader.read_u8()?;
        let _summon_owner = reader.read_u32()?;
        let _summon_skill_index = reader.read_u16()?;

        let direction = reader.read_f32()?;
        for _ in 0..5 {
            let _event_status = reader.read_u16()?;
        }

        for _ in 0.._status_count {
            let _state_index = reader.read_u16()?;
            let _source = reader.read_u8()?;
            let _adjust_value = reader.read_i32()?;
            let _cast_adjust_value = reader.read_i32()?;
            let _expire_time = reader.read_i32()?;
            let _skill_index = reader.read_u16()?;
        }

        Ok(Self {
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
            status_effects: ActiveStatusEffects::default(),
        })
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
        let _path_destination_x = reader.read_f32()?;
        let _path_destination_y = reader.read_f32()?;
        let _waypoint_destination_x = reader.read_f32()?;
        let _waypoint_destination_y = reader.read_f32()?;
        let _is_path_end = reader.read_u8()?;
        let _skill_index = reader.read_u16()?;
        let command = reader.read_command_state()?;
        let target_entity_id = reader.read_option_entity_id()?;
        let _passenger_entity_id = reader.read_option_entity_id()?;
        let move_mode = reader.read_move_mode_u8()?;
        let health = HealthPoints::new(reader.read_i32()?);
        let _max_health = reader.read_i32()?;
        let _mana = reader.read_i32()?;
        let _max_mana = reader.read_i32()?;
        let _attack_range_rate = reader.read_i32()?;
        let _attack_range_value = reader.read_i32()?;
        let _skill_range_rate = reader.read_i32()?;
        let _skill_range_value = reader.read_i32()?;
        let team = Team::new(reader.read_u32()?);
        let _sub_flags = reader.read_u64()?;
        let _status_count = reader.read_u16()?;

        let npc_id = reader.read_i16()?;
        let _npc_is_visible = npc_id > 0;
        let npc_id = NpcId::new(npc_id.unsigned_abs()).ok_or(PacketError::InvalidPacket)?;
        let quest_index = reader.read_u16()?;
        let _skill_motion = reader.read_u8()?;
        let _summon_owner = reader.read_u32()?;
        let _summon_skill_index = reader.read_u16()?;

        for _ in 0.._status_count {
            let _state_index = reader.read_u16()?;
            let _source = reader.read_u8()?;
            let _adjust_value = reader.read_i32()?;
            let _cast_adjust_value = reader.read_i32()?;
            let _expire_time = reader.read_i32()?;
            let _skill_index = reader.read_u16()?;
        }

        Ok(Self {
            entity_id,
            npc: Npc::new(npc_id, quest_index),
            position,
            team,
            destination,
            command,
            target_entity_id,
            health,
            move_mode,
            status_effects: ActiveStatusEffects::default(),
        })
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
        let _ori_run_speed = reader.read_u16()? as i32;
        let run_speed = reader.read_u16()? as i32;
        let _ori_attack_speed = reader.read_i32()?;
        let passive_attack_speed = reader.read_i32()?;
        let _weight_rate = reader.read_u8()?;

        Ok(Self {
            entity_id,
            run_speed,
            passive_attack_speed,
        })
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
        let zone_id = ZoneId::new(reader.read_u32()? as u16).ok_or(PacketError::InvalidPacket)?;
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let run_mode = reader.read_u8()?;
        let ride_mode = reader.read_u8()?;

        // tagObjVAR
        let _next_check_time = reader.read_u32()?;
        let _next_trigger = reader.read_u32()?;
        let _pass_time = reader.read_u32()?;

        for _ in 0..20 {
            let _ai_var_n = reader.read_u16()?;
        }

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
