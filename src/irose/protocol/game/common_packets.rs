use modular_bitfield::prelude::*;
use std::convert::TryInto;

use crate::{
    data::{
        Damage, EquipmentItem, Item, ItemReference, SkillPageType, StackableItem, StatusEffectType,
    },
    game::components::{
        AmmoIndex, ClientEntityId, Equipment, EquipmentIndex, HotbarSlot, InventoryPageType,
        ItemSlot, Money, MoveMode, SkillSlot, StatusEffects, VehiclePartIndex,
    },
    irose::data::{decode_item_type, encode_item_type},
    protocol::{PacketReader, PacketWriter, ProtocolError},
};

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketHotbarSlot {
    slot_type: B5,
    index: B11,
}

pub trait PacketReadHotbarSlot {
    fn read_hotbar_slot(&mut self) -> Result<Option<HotbarSlot>, ProtocolError>;
}

impl<'a> PacketReadHotbarSlot for PacketReader<'a> {
    fn read_hotbar_slot(&mut self) -> Result<Option<HotbarSlot>, ProtocolError> {
        let slot =
            PacketHotbarSlot::from_bytes(self.read_fixed_length_bytes(2)?.try_into().unwrap());
        match slot.slot_type() {
            1 => Ok(Some(HotbarSlot::Inventory(slot.index()))),
            2 => Ok(Some(HotbarSlot::Command(slot.index()))),
            3 => Ok(Some(HotbarSlot::Skill(slot.index()))),
            4 => Ok(Some(HotbarSlot::Emote(slot.index()))),
            5 => Ok(Some(HotbarSlot::Dialog(slot.index()))),
            6 => Ok(Some(HotbarSlot::ClanSkill(slot.index()))),
            _ => Ok(None),
        }
    }
}

pub trait PacketWriteHotbarSlot {
    fn write_hotbar_slot(&mut self, slot: &Option<HotbarSlot>);
}

impl PacketWriteHotbarSlot for PacketWriter {
    fn write_hotbar_slot(&mut self, slot: &Option<HotbarSlot>) {
        let (slot_type, index) = match slot {
            Some(HotbarSlot::Inventory(index)) => (1, *index),
            Some(HotbarSlot::Command(index)) => (2, *index),
            Some(HotbarSlot::Skill(index)) => (3, *index),
            Some(HotbarSlot::Emote(index)) => (4, *index),
            Some(HotbarSlot::Dialog(index)) => (5, *index),
            Some(HotbarSlot::ClanSkill(index)) => (6, *index),
            _ => (0, 0),
        };
        let slot = PacketHotbarSlot::new()
            .with_slot_type(slot_type)
            .with_index(index);
        self.write_bytes(&slot.into_bytes());
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentAmmoPart {
    #[skip(getters)]
    pub item_type: B5,
    #[skip(getters)]
    pub item_number: B10,
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
pub struct PacketFullItemHeader {
    #[skip(setters)]
    item_type: B5,
    #[skip(setters)]
    item_number: B10,
    #[skip]
    __: B1,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentItemFull {
    #[skip(getters)]
    item_type: B5,
    #[skip(getters)]
    item_number: B10,
    is_crafted: bool,
    gem: B9,
    durability: B7,
    life: B10,
    has_socket: bool,
    is_appraised: bool,
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
    quantity: B32,
}

pub trait PacketReadItems {
    fn read_item_full(&mut self) -> Result<Option<Item>, ProtocolError>;
}

impl<'a> PacketReadItems for PacketReader<'a> {
    fn read_item_full(&mut self) -> Result<Option<Item>, ProtocolError> {
        let item_bytes = self.read_fixed_length_bytes(6)?;
        let item_header = PacketFullItemHeader::from_bytes(item_bytes[0..2].try_into().unwrap());
        let item_number = item_header.item_number();
        if item_number == 0 || item_number > 999 {
            return Ok(None);
        }

        if let Some(item_type) = decode_item_type(item_header.item_type() as usize) {
            let item_reference = ItemReference::new(item_type, item_header.item_number() as usize);

            if item_type.is_stackable_item() {
                let stackable_item =
                    PacketStackableItemFull::from_bytes(item_bytes.try_into().unwrap());
                if let Some(item) = StackableItem::new(&item_reference, stackable_item.quantity()) {
                    return Ok(Some(Item::Stackable(item)));
                }
            } else {
                let equipment_item =
                    PacketEquipmentItemFull::from_bytes(item_bytes.try_into().unwrap());
                if let Some(mut item) = EquipmentItem::new(&item_reference) {
                    item.gem = equipment_item.gem();
                    item.durability = equipment_item.durability();
                    item.life = equipment_item.life();
                    item.grade = equipment_item.grade();
                    item.is_crafted = equipment_item.is_crafted();
                    item.has_socket = equipment_item.has_socket();
                    item.is_appraised = equipment_item.is_appraised();
                    return Ok(Some(Item::Equipment(item)));
                }
            }
        }

        Ok(None)
    }
}

pub trait PacketWriteItems {
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
            if let Some(item_type) = encode_item_type(item.item.item_type) {
                let part = PacketEquipmentAmmoPart::new()
                    .with_item_number(item.item.item_number as u16)
                    .with_item_type(item_type as u8);
                for b in part.into_bytes().iter() {
                    self.write_u8(*b);
                }
            } else {
                self.write_u16(0);
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
                if let Some(item_type) = encode_item_type(equipment.item.item_type) {
                    let item = PacketEquipmentItemFull::new()
                        .with_item_type(item_type as u8)
                        .with_item_number(equipment.item.item_number as u16)
                        .with_is_crafted(equipment.is_crafted)
                        .with_gem(equipment.gem)
                        .with_durability(equipment.durability)
                        .with_life(equipment.life)
                        .with_has_socket(equipment.has_socket)
                        .with_is_appraised(equipment.is_appraised)
                        .with_grade(equipment.grade);
                    self.write_bytes(&item.into_bytes());
                } else {
                    self.write_u16(0);
                    self.write_u32(0);
                }
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
                if let Some(item_type) = encode_item_type(stackable.item.item_type) {
                    let item = PacketStackableItemFull::new()
                        .with_item_type(item_type as u8)
                        .with_item_number(stackable.item.item_number as u16)
                        .with_quantity(stackable.quantity);
                    self.write_bytes(&item.into_bytes());
                } else {
                    self.write_u16(0);
                    self.write_u32(0);
                }
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

const EQUIPMENT_START_INDEX: usize = 1;
const EQUIPMENT_END_INDEX: usize = 12;

const INVENTORY_PAGE_SIZE: usize = 5 * 6;
const INVENTORY_PAGES: usize = 4;
const INVENTORY_START_INDEX: usize = EQUIPMENT_END_INDEX;
const INVENTORY_END_INDEX: usize = INVENTORY_START_INDEX + INVENTORY_PAGE_SIZE * INVENTORY_PAGES;

const AMMO_START_INDEX: usize = INVENTORY_END_INDEX;
const AMMO_END_INDEX: usize = AMMO_START_INDEX + 3;

const VEHICLE_START_INDEX: usize = AMMO_END_INDEX;
const VEHICLE_END_INDEX: usize = VEHICLE_START_INDEX + 4;

pub trait PacketReadEquipmentIndex {
    fn read_equipment_index_u16(&mut self) -> Result<EquipmentIndex, ProtocolError>;
}

pub trait PacketWriteEquipmentIndex {
    fn write_equipment_index_u16(&mut self, equipment_index: EquipmentIndex);
}

pub fn decode_ammo_index(index: usize) -> Option<AmmoIndex> {
    match index {
        0 => Some(AmmoIndex::Arrow),
        1 => Some(AmmoIndex::Bullet),
        2 => Some(AmmoIndex::Throw),
        _ => None,
    }
}

pub fn decode_equipment_index(index: usize) -> Option<EquipmentIndex> {
    match index {
        1 => Some(EquipmentIndex::Face),
        2 => Some(EquipmentIndex::Head),
        3 => Some(EquipmentIndex::Body),
        4 => Some(EquipmentIndex::Back),
        5 => Some(EquipmentIndex::Hands),
        6 => Some(EquipmentIndex::Feet),
        7 => Some(EquipmentIndex::WeaponRight),
        8 => Some(EquipmentIndex::WeaponLeft),
        9 => Some(EquipmentIndex::Necklace),
        10 => Some(EquipmentIndex::Ring),
        11 => Some(EquipmentIndex::Earring),
        _ => None,
    }
}

pub fn encode_ammo_index(index: AmmoIndex) -> usize {
    match index {
        AmmoIndex::Arrow => 0,
        AmmoIndex::Bullet => 1,
        AmmoIndex::Throw => 2,
    }
}

fn encode_equipment_index(index: EquipmentIndex) -> usize {
    match index {
        EquipmentIndex::Face => 1,
        EquipmentIndex::Head => 2,
        EquipmentIndex::Body => 3,
        EquipmentIndex::Back => 4,
        EquipmentIndex::Hands => 5,
        EquipmentIndex::Feet => 6,
        EquipmentIndex::WeaponRight => 7,
        EquipmentIndex::WeaponLeft => 8,
        EquipmentIndex::Necklace => 9,
        EquipmentIndex::Ring => 10,
        EquipmentIndex::Earring => 11,
    }
}

impl<'a> PacketReadEquipmentIndex for PacketReader<'a> {
    fn read_equipment_index_u16(&mut self) -> Result<EquipmentIndex, ProtocolError> {
        decode_equipment_index(self.read_u16()? as usize).ok_or(ProtocolError::InvalidPacket)
    }
}

impl PacketWriteEquipmentIndex for PacketWriter {
    fn write_equipment_index_u16(&mut self, equipment_index: EquipmentIndex) {
        self.write_u16(encode_equipment_index(equipment_index) as u16)
    }
}

pub trait PacketReadVehiclePartIndex {
    fn read_vehicle_part_index_u16(&mut self) -> Result<VehiclePartIndex, ProtocolError>;
}

pub trait PacketWriteVehiclePartIndex {
    fn write_vehicle_part_index_u16(&mut self, vehicle_part_index: VehiclePartIndex);
}

fn decode_vehicle_part_index(index: usize) -> Option<VehiclePartIndex> {
    match index {
        0 => Some(VehiclePartIndex::Body),
        1 => Some(VehiclePartIndex::Engine),
        2 => Some(VehiclePartIndex::Leg),
        3 => Some(VehiclePartIndex::Ability),
        4 => Some(VehiclePartIndex::Arms),
        _ => None,
    }
}

fn encode_vehicle_part_index(index: VehiclePartIndex) -> usize {
    match index {
        VehiclePartIndex::Body => 0,
        VehiclePartIndex::Engine => 1,
        VehiclePartIndex::Leg => 2,
        VehiclePartIndex::Ability => 3,
        VehiclePartIndex::Arms => 4,
    }
}

impl<'a> PacketReadVehiclePartIndex for PacketReader<'a> {
    fn read_vehicle_part_index_u16(&mut self) -> Result<VehiclePartIndex, ProtocolError> {
        decode_vehicle_part_index(self.read_u16()? as usize).ok_or(ProtocolError::InvalidPacket)
    }
}

impl PacketWriteVehiclePartIndex for PacketWriter {
    fn write_vehicle_part_index_u16(&mut self, vehicle_part_index: VehiclePartIndex) {
        self.write_u16(encode_vehicle_part_index(vehicle_part_index) as u16)
    }
}

pub trait PacketReadItemSlot {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, ProtocolError>;
    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, ProtocolError>;
}

pub trait PacketWriteItemSlot {
    fn write_item_slot_u8(&mut self, item_slot: ItemSlot);
    fn write_item_slot_u16(&mut self, item_slot: ItemSlot);
}

pub fn decode_item_slot(index: usize) -> Option<ItemSlot> {
    if index == 0 {
        None
    } else if (EQUIPMENT_START_INDEX..EQUIPMENT_END_INDEX).contains(&index) {
        decode_equipment_index(index).map(ItemSlot::Equipment)
    } else if (INVENTORY_START_INDEX..INVENTORY_END_INDEX).contains(&index) {
        let index = index - INVENTORY_START_INDEX;
        let page = index / INVENTORY_PAGE_SIZE;
        let slot = index % INVENTORY_PAGE_SIZE;
        match page {
            0 => Some(ItemSlot::Inventory(InventoryPageType::Equipment, slot)),
            1 => Some(ItemSlot::Inventory(InventoryPageType::Consumables, slot)),
            2 => Some(ItemSlot::Inventory(InventoryPageType::Materials, slot)),
            3 => Some(ItemSlot::Inventory(InventoryPageType::Vehicles, slot)),
            _ => None,
        }
    } else if (AMMO_START_INDEX..AMMO_END_INDEX).contains(&index) {
        decode_ammo_index(index - AMMO_START_INDEX).map(ItemSlot::Ammo)
    } else if (VEHICLE_START_INDEX..VEHICLE_END_INDEX).contains(&index) {
        match index - VEHICLE_START_INDEX {
            0 => Some(ItemSlot::Vehicle(VehiclePartIndex::Body)),
            1 => Some(ItemSlot::Vehicle(VehiclePartIndex::Engine)),
            2 => Some(ItemSlot::Vehicle(VehiclePartIndex::Leg)),
            3 => Some(ItemSlot::Vehicle(VehiclePartIndex::Ability)),
            _ => None,
        }
    } else {
        None
    }
}

fn encode_item_slot(slot: ItemSlot) -> usize {
    match slot {
        ItemSlot::Equipment(equipment_index) => encode_equipment_index(equipment_index),
        ItemSlot::Inventory(page_type, index) => match page_type {
            InventoryPageType::Equipment => INVENTORY_START_INDEX + index,
            InventoryPageType::Consumables => INVENTORY_START_INDEX + INVENTORY_PAGE_SIZE + index,
            InventoryPageType::Materials => INVENTORY_START_INDEX + 2 * INVENTORY_PAGE_SIZE + index,
            InventoryPageType::Vehicles => INVENTORY_START_INDEX + 3 * INVENTORY_PAGE_SIZE + index,
        },
        ItemSlot::Ammo(AmmoIndex::Arrow) => AMMO_START_INDEX,
        ItemSlot::Ammo(AmmoIndex::Bullet) => AMMO_START_INDEX + 1,
        ItemSlot::Ammo(AmmoIndex::Throw) => AMMO_START_INDEX + 2,
        ItemSlot::Vehicle(VehiclePartIndex::Body) => VEHICLE_START_INDEX,
        ItemSlot::Vehicle(VehiclePartIndex::Engine) => VEHICLE_START_INDEX + 1,
        ItemSlot::Vehicle(VehiclePartIndex::Leg) => VEHICLE_START_INDEX + 2,
        ItemSlot::Vehicle(VehiclePartIndex::Ability) => VEHICLE_START_INDEX + 3,
        ItemSlot::Vehicle(VehiclePartIndex::Arms) => VEHICLE_START_INDEX + 4,
    }
}

impl<'a> PacketReadItemSlot for PacketReader<'a> {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, ProtocolError> {
        decode_item_slot(self.read_u8()? as usize).ok_or(ProtocolError::InvalidPacket)
    }

    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, ProtocolError> {
        decode_item_slot(self.read_u16()? as usize).ok_or(ProtocolError::InvalidPacket)
    }
}

impl PacketWriteItemSlot for PacketWriter {
    fn write_item_slot_u8(&mut self, item_slot: ItemSlot) {
        self.write_u8(encode_item_slot(item_slot) as u8)
    }

    fn write_item_slot_u16(&mut self, item_slot: ItemSlot) {
        self.write_u16(encode_item_slot(item_slot) as u16)
    }
}

pub trait PacketReadSkillSlot {
    fn read_skill_slot_u8(&mut self) -> Result<SkillSlot, ProtocolError>;
}

pub trait PacketWriteSkillSlot {
    fn write_skill_slot_u8(&mut self, slot: SkillSlot);
}

fn skill_slot_from_index(index: usize) -> Result<SkillSlot, ProtocolError> {
    match index {
        0..=29 => Ok(SkillSlot(SkillPageType::Basic, index)),
        30..=59 => Ok(SkillSlot(SkillPageType::Active, index - 30)),
        60..=89 => Ok(SkillSlot(SkillPageType::Passive, index - 60)),
        90..=119 => Ok(SkillSlot(SkillPageType::Clan, index - 90)),
        _ => Err(ProtocolError::InvalidPacket),
    }
}

fn skill_slot_to_index(slot: SkillSlot) -> usize {
    match slot {
        SkillSlot(SkillPageType::Basic, index) => index,
        SkillSlot(SkillPageType::Active, index) => 30 + index,
        SkillSlot(SkillPageType::Passive, index) => (2 * 30) + index,
        SkillSlot(SkillPageType::Clan, index) => (3 * 30) + index,
    }
}

impl<'a> PacketReadSkillSlot for PacketReader<'a> {
    fn read_skill_slot_u8(&mut self) -> Result<SkillSlot, ProtocolError> {
        skill_slot_from_index(self.read_u8()? as usize)
    }
}

impl PacketWriteSkillSlot for PacketWriter {
    fn write_skill_slot_u8(&mut self, slot: SkillSlot) {
        self.write_u8(skill_slot_to_index(slot) as u8)
    }
}

pub trait PacketWriteMoveMode {
    fn write_move_mode_u8(&mut self, move_mode: MoveMode);
    fn write_move_mode_toggle_u8(&mut self, move_mode: MoveMode);
}

impl PacketWriteMoveMode for PacketWriter {
    fn write_move_mode_u8(&mut self, move_mode: MoveMode) {
        self.write_u8(match move_mode {
            MoveMode::Walk => 0,
            MoveMode::Run => 1,
            MoveMode::Drive => 2,
        })
    }

    fn write_move_mode_toggle_u8(&mut self, move_mode: MoveMode) {
        self.write_u8(match move_mode {
            MoveMode::Walk => 2,
            MoveMode::Run => 3,
            MoveMode::Drive => 4,
        })
    }
}

pub trait PacketWriteStatusEffects {
    fn write_status_effects_flags_u32(&mut self, status_effects: &StatusEffects);
    fn write_status_effects_values(&mut self, status_effects: &StatusEffects);
}

fn get_status_effect_type_flag(status_effect_type: StatusEffectType) -> u32 {
    match status_effect_type {
        StatusEffectType::IncreaseHp => 0x00000001,
        StatusEffectType::IncreaseMp => 0x00000002,
        StatusEffectType::Poisoned => 0x00000004,
        StatusEffectType::IncreaseMaxHp => 0x00000010,
        StatusEffectType::IncreaseMaxMp => 0x00000020,
        StatusEffectType::IncreaseMoveSpeed => 0x00000040,
        StatusEffectType::DecreaseMoveSpeed => 0x00000080,
        StatusEffectType::IncreaseAttackSpeed => 0x00000100,
        StatusEffectType::DecreaseAttackSpeed => 0x00000200,
        StatusEffectType::IncreaseAttackPower => 0x00000400,
        StatusEffectType::DecreaseAttackPower => 0x00000800,
        StatusEffectType::IncreaseDefence => 0x00001000,
        StatusEffectType::DecreaseDefence => 0x00002000,
        StatusEffectType::IncreaseResistance => 0x00004000,
        StatusEffectType::DecreaseResistance => 0x00008000,
        StatusEffectType::IncreaseHit => 0x00010000,
        StatusEffectType::DecreaseHit => 0x00020000,
        StatusEffectType::IncreaseCritical => 0x00040000,
        StatusEffectType::DecreaseCritical => 0x00080000,
        StatusEffectType::IncreaseAvoid => 0x00100000,
        StatusEffectType::DecreaseAvoid => 0x00200000,
        StatusEffectType::Dumb => 0x00400000,
        StatusEffectType::Sleep => 0x00800000,
        StatusEffectType::Fainting => 0x01000000,
        StatusEffectType::Disguise => 0x02000000,
        StatusEffectType::Transparent => 0x04000000,
        StatusEffectType::ShieldDamage => 0x08000000,
        StatusEffectType::AdditionalDamageRate => 0x10000000,
        StatusEffectType::DecreaseLifeTime => 0x20000000,
        StatusEffectType::Revive => 0x40000000,
        StatusEffectType::Taunt => 0x80000000,
        _ => 0,
    }
}

impl PacketWriteStatusEffects for PacketWriter {
    fn write_status_effects_flags_u32(&mut self, status_effects: &StatusEffects) {
        let mut status_effect_flags = 0u32;

        for (status_effect_type, status_effect) in status_effects.active.iter() {
            if status_effect.is_some() {
                status_effect_flags |= get_status_effect_type_flag(status_effect_type);
            }
        }

        self.write_u32(status_effect_flags);
    }

    fn write_status_effects_values(&mut self, status_effects: &StatusEffects) {
        if let Some(value) = status_effects.get_status_effect_value(StatusEffectType::IncreaseMaxHp)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            status_effects.get_status_effect_value(StatusEffectType::IncreaseMoveSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            status_effects.get_status_effect_value(StatusEffectType::DecreaseMoveSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            status_effects.get_status_effect_value(StatusEffectType::IncreaseAttackSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            status_effects.get_status_effect_value(StatusEffectType::DecreaseAttackSpeed)
        {
            self.write_u16(value as u16);
        }
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

pub trait PacketWriteDamage {
    fn write_damage_u16(&mut self, damage: &Damage, is_killed: bool);
}

impl PacketWriteDamage for PacketWriter {
    fn write_damage_u16(&mut self, damage: &Damage, is_killed: bool) {
        let mut action = 0u8;

        if damage.is_critical {
            action |= 0x08;
        }

        if damage.apply_hit_stun {
            action |= 0x04;
        }

        if is_killed {
            action |= 0x10;
        }

        let damage = PacketServerDamage::new()
            .with_amount(damage.amount as u16)
            .with_action(action);

        for b in damage.into_bytes().iter() {
            self.write_u8(*b);
        }
    }
}

pub trait PacketWriteEntityId {
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
