use modular_bitfield::prelude::*;
use std::convert::TryInto;

use rose_data::{
    AmmoIndex, EquipmentIndex, EquipmentItem, Item, ItemReference, ItemType, StackableItem,
    StatusEffectId, StatusEffectType, VehiclePartIndex,
};
use rose_data_irose::{
    decode_ammo_index, decode_equipment_index, decode_item_type, decode_vehicle_part_index,
    encode_equipment_index, encode_item_type, encode_vehicle_part_index, SKILL_PAGE_SIZE,
};
use rose_game_common::{
    components::{
        ActiveStatusEffect, CharacterGender, ClanMark, ClanMemberPosition, Equipment, HealthPoints,
        HotbarSlot, InventoryPageType, ItemSlot, Money, MoveMode, SkillSlot, Stamina,
    },
    data::Damage,
    messages::{
        server::{
            ActiveStatusEffects, CommandState, PartyMemberInfo, PartyMemberInfoOffline,
            PartyMemberInfoOnline,
        },
        ClientEntityId, PartyItemSharing, PartyXpSharing,
    },
};
use rose_network_common::{PacketError, PacketReader, PacketWriter};

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketHotbarSlot {
    slot_type: B5,
    index: B11,
}

pub trait PacketReadHotbarSlot {
    fn read_hotbar_slot(&mut self) -> Result<Option<HotbarSlot>, PacketError>;
}

impl<'a> PacketReadHotbarSlot for PacketReader<'a> {
    fn read_hotbar_slot(&mut self) -> Result<Option<HotbarSlot>, PacketError> {
        let slot =
            PacketHotbarSlot::from_bytes(self.read_fixed_length_bytes(2)?.try_into().unwrap());
        match slot.slot_type() {
            1 => Ok(Some(HotbarSlot::Inventory(
                decode_item_slot(slot.index() as usize).ok_or(PacketError::InvalidPacket)?,
            ))),
            2 => Ok(Some(HotbarSlot::Command(slot.index()))),
            3 => Ok(Some(HotbarSlot::Skill(decode_skill_slot(
                slot.index() as usize
            )?))),
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
            &Some(HotbarSlot::Inventory(item_slot)) => (1, encode_item_slot(item_slot) as u16),
            Some(HotbarSlot::Command(index)) => (2, *index),
            &Some(HotbarSlot::Skill(skill_slot)) => (3, encode_skill_slot(skill_slot) as u16),
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

pub trait PacketReadCharacterGender {
    fn read_character_gender_u8(&mut self) -> Result<CharacterGender, PacketError>;
}

impl<'a> PacketReadCharacterGender for PacketReader<'a> {
    fn read_character_gender_u8(&mut self) -> Result<CharacterGender, PacketError> {
        match self.read_u8()? {
            0 => Ok(CharacterGender::Male),
            1 => Ok(CharacterGender::Female),
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

pub trait PacketWriteCharacterGender {
    fn write_character_gender_u8(&mut self, gender: CharacterGender);
}

impl PacketWriteCharacterGender for PacketWriter {
    fn write_character_gender_u8(&mut self, gender: CharacterGender) {
        match gender {
            CharacterGender::Male => self.write_u8(0),
            CharacterGender::Female => self.write_u8(1),
        }
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentAmmoPart {
    pub item_type: B5,
    pub item_number: B10,
    #[skip]
    __: B1,
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketEquipmentItemPart {
    item_number: B10,
    gem: B9,
    has_socket: bool,
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
    item_type: B5,
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
    item_type: B5,
    item_number: B10,
    #[skip]
    __: B1,
    quantity: B32,
}

pub trait PacketReadItems {
    fn read_item_full(&mut self) -> Result<Option<Item>, PacketError>;
    fn read_item_or_money_full(&mut self) -> Result<(Option<Item>, Option<Money>), PacketError>;
    fn read_equipment_item_full(&mut self) -> Result<Option<EquipmentItem>, PacketError>;
    fn read_stackable_item_full(&mut self) -> Result<Option<StackableItem>, PacketError>;
    fn read_equipment_visible_part(&mut self) -> Result<Equipment, PacketError>;
    fn read_equipment_item_part(
        &mut self,
        item_type: ItemType,
    ) -> Result<Option<EquipmentItem>, PacketError>;
    fn read_equipment_ammo_part(&mut self) -> Result<Option<StackableItem>, PacketError>;
}

fn decode_item_full_bytes(item_bytes: &[u8]) -> Result<Option<Item>, PacketError> {
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
            if let Some(item) = StackableItem::new(item_reference, stackable_item.quantity()) {
                return Ok(Some(Item::Stackable(item)));
            }
        } else {
            let equipment_item =
                PacketEquipmentItemFull::from_bytes(item_bytes.try_into().unwrap());
            if let Some(mut item) = EquipmentItem::new(item_reference, equipment_item.durability())
            {
                item.gem = equipment_item.gem();
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

impl<'a> PacketReadItems for PacketReader<'a> {
    fn read_item_full(&mut self) -> Result<Option<Item>, PacketError> {
        let item_bytes = self.read_fixed_length_bytes(6)?;
        decode_item_full_bytes(item_bytes)
    }

    fn read_item_or_money_full(&mut self) -> Result<(Option<Item>, Option<Money>), PacketError> {
        let item_bytes = self.read_fixed_length_bytes(6)?;
        let stackable_item = PacketStackableItemFull::from_bytes(item_bytes.try_into().unwrap());
        if stackable_item.item_type() == 31 {
            if stackable_item.quantity() == 0 {
                Ok((None, None))
            } else {
                Ok((None, Some(Money(stackable_item.quantity() as i64))))
            }
        } else {
            decode_item_full_bytes(item_bytes).map_or(Ok((None, None)), |item| Ok((item, None)))
        }
    }

    fn read_equipment_item_full(&mut self) -> Result<Option<EquipmentItem>, PacketError> {
        let item = PacketEquipmentItemFull::from_bytes(
            self.read_fixed_length_bytes(6)?.try_into().unwrap(),
        );

        if let Some(item_type) = decode_item_type(item.item_type() as usize) {
            if let Some(mut equipment) = EquipmentItem::new(
                ItemReference::new(item_type, item.item_number() as usize),
                item.durability(),
            ) {
                equipment.is_crafted = item.is_crafted();
                equipment.gem = item.gem();
                equipment.life = item.life();
                equipment.has_socket = item.has_socket();
                equipment.is_appraised = item.is_appraised();
                equipment.grade = item.grade();
                return Ok(Some(equipment));
            }
        }

        Ok(None)
    }

    fn read_stackable_item_full(&mut self) -> Result<Option<StackableItem>, PacketError> {
        let item = PacketStackableItemFull::from_bytes(
            self.read_fixed_length_bytes(6)?.try_into().unwrap(),
        );

        if let Some(item_type) = decode_item_type(item.item_type() as usize) {
            return Ok(StackableItem::new(
                ItemReference::new(item_type, item.item_number() as usize),
                item.quantity(),
            ));
        }

        Ok(None)
    }

    fn read_equipment_item_part(
        &mut self,
        item_type: ItemType,
    ) -> Result<Option<EquipmentItem>, PacketError> {
        let item_part = PacketEquipmentItemPart::from_bytes(
            self.read_fixed_length_bytes(3)?.try_into().unwrap(),
        );
        self.read_u8()?;

        if let Some(mut item) = EquipmentItem::new(
            ItemReference::new(item_type, item_part.item_number() as usize),
            0,
        ) {
            item.gem = item_part.gem();
            item.grade = item_part.grade();
            item.has_socket = item_part.has_socket();
            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    fn read_equipment_visible_part(&mut self) -> Result<Equipment, PacketError> {
        let mut equipment = Equipment::default();

        for index in [
            EquipmentIndex::Head,
            EquipmentIndex::Body,
            EquipmentIndex::Hands,
            EquipmentIndex::Feet,
            EquipmentIndex::Face,
            EquipmentIndex::Back,
            EquipmentIndex::Weapon,
            EquipmentIndex::SubWeapon,
        ] {
            equipment.equipped_items[index] = self.read_equipment_item_part(index.into())?;
        }

        Ok(equipment)
    }

    fn read_equipment_ammo_part(&mut self) -> Result<Option<StackableItem>, PacketError> {
        let ammo_part = PacketEquipmentAmmoPart::from_bytes(
            self.read_fixed_length_bytes(2)?.try_into().unwrap(),
        );

        if let Some(item_type) = decode_item_type(ammo_part.item_type() as usize) {
            if let Some(item) = StackableItem::new(
                ItemReference::new(item_type, ammo_part.item_number() as usize),
                999,
            ) {
                Ok(Some(item))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
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
            EquipmentIndex::Weapon,
            EquipmentIndex::SubWeapon,
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
    fn read_equipment_index_u8(&mut self) -> Result<EquipmentIndex, PacketError>;
    fn read_equipment_index_u16(&mut self) -> Result<EquipmentIndex, PacketError>;
}

pub trait PacketWriteEquipmentIndex {
    fn write_equipment_index_u8(&mut self, equipment_index: EquipmentIndex);
    fn write_equipment_index_u16(&mut self, equipment_index: EquipmentIndex);
}

impl<'a> PacketReadEquipmentIndex for PacketReader<'a> {
    fn read_equipment_index_u8(&mut self) -> Result<EquipmentIndex, PacketError> {
        decode_equipment_index(self.read_u8()? as usize).ok_or(PacketError::InvalidPacket)
    }

    fn read_equipment_index_u16(&mut self) -> Result<EquipmentIndex, PacketError> {
        decode_equipment_index(self.read_u16()? as usize).ok_or(PacketError::InvalidPacket)
    }
}

impl PacketWriteEquipmentIndex for PacketWriter {
    fn write_equipment_index_u8(&mut self, equipment_index: EquipmentIndex) {
        self.write_u8(encode_equipment_index(equipment_index).unwrap_or(0) as u8)
    }

    fn write_equipment_index_u16(&mut self, equipment_index: EquipmentIndex) {
        self.write_u16(encode_equipment_index(equipment_index).unwrap_or(0) as u16)
    }
}

pub trait PacketReadVehiclePartIndex {
    fn read_vehicle_part_index_u16(&mut self) -> Result<VehiclePartIndex, PacketError>;
}

pub trait PacketWriteVehiclePartIndex {
    fn write_vehicle_part_index_u16(&mut self, vehicle_part_index: VehiclePartIndex);
}

impl<'a> PacketReadVehiclePartIndex for PacketReader<'a> {
    fn read_vehicle_part_index_u16(&mut self) -> Result<VehiclePartIndex, PacketError> {
        decode_vehicle_part_index(self.read_u16()? as usize).ok_or(PacketError::InvalidPacket)
    }
}

impl PacketWriteVehiclePartIndex for PacketWriter {
    fn write_vehicle_part_index_u16(&mut self, vehicle_part_index: VehiclePartIndex) {
        self.write_u16(encode_vehicle_part_index(vehicle_part_index).unwrap_or(0) as u16)
    }
}

pub trait PacketReadItemSlot {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, PacketError>;
    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, PacketError>;
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
        decode_vehicle_part_index(index - VEHICLE_START_INDEX).map(ItemSlot::Vehicle)
    } else {
        None
    }
}

pub fn encode_item_slot(slot: ItemSlot) -> usize {
    match slot {
        ItemSlot::Equipment(equipment_index) => {
            encode_equipment_index(equipment_index).unwrap_or(0)
        }
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
        ItemSlot::Vehicle(VehiclePartIndex::Arms) => VEHICLE_START_INDEX + 3,
    }
}

impl<'a> PacketReadItemSlot for PacketReader<'a> {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, PacketError> {
        decode_item_slot(self.read_u8()? as usize).ok_or(PacketError::InvalidPacket)
    }

    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, PacketError> {
        decode_item_slot(self.read_u16()? as usize).ok_or(PacketError::InvalidPacket)
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
    fn read_skill_slot_u8(&mut self) -> Result<SkillSlot, PacketError>;
}

pub trait PacketWriteSkillSlot {
    fn write_skill_slot_u8(&mut self, slot: SkillSlot);
}

fn decode_skill_slot(index: usize) -> Result<SkillSlot, PacketError> {
    let page = index / SKILL_PAGE_SIZE;
    let slot = index % SKILL_PAGE_SIZE;
    Ok(SkillSlot(page, slot))
}

fn encode_skill_slot(slot: SkillSlot) -> usize {
    slot.0 * SKILL_PAGE_SIZE + slot.1
}

impl<'a> PacketReadSkillSlot for PacketReader<'a> {
    fn read_skill_slot_u8(&mut self) -> Result<SkillSlot, PacketError> {
        decode_skill_slot(self.read_u8()? as usize)
    }
}

impl PacketWriteSkillSlot for PacketWriter {
    fn write_skill_slot_u8(&mut self, slot: SkillSlot) {
        self.write_u8(encode_skill_slot(slot) as u8)
    }
}

pub trait PacketReadMoveMode {
    fn read_move_mode_u8(&mut self) -> Result<MoveMode, PacketError>;
}

impl<'a> PacketReadMoveMode for PacketReader<'a> {
    fn read_move_mode_u8(&mut self) -> Result<MoveMode, PacketError> {
        match self.read_u8()? {
            0 => Ok(MoveMode::Walk),
            1 => Ok(MoveMode::Run),
            2 => Ok(MoveMode::Drive),
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

pub trait PacketWriteMoveMode {
    fn write_move_mode_u8(&mut self, move_mode: MoveMode);
}

impl PacketWriteMoveMode for PacketWriter {
    fn write_move_mode_u8(&mut self, move_mode: MoveMode) {
        self.write_u8(match move_mode {
            MoveMode::Walk => 0,
            MoveMode::Run => 1,
            MoveMode::Drive => 2,
        })
    }
}

pub trait PacketReadStatusEffects {
    fn read_status_effects_flags_u32(
        &mut self,
        status_effects: &mut ActiveStatusEffects,
    ) -> Result<(), PacketError>;

    fn read_status_effects_values(
        &mut self,
        status_effects: &mut ActiveStatusEffects,
    ) -> Result<(), PacketError>;
}

impl<'a> PacketReadStatusEffects for PacketReader<'a> {
    fn read_status_effects_flags_u32(
        &mut self,
        status_effects: &mut ActiveStatusEffects,
    ) -> Result<(), PacketError> {
        let flags = self.read_u32()?;

        for (status_effect_type, status_effect) in status_effects.iter_mut() {
            if (flags & get_status_effect_type_flag(status_effect_type)) != 0 {
                *status_effect = Some(ActiveStatusEffect {
                    id: StatusEffectId::new(1).unwrap(),
                    value: 0,
                });
            }
        }

        Ok(())
    }

    fn read_status_effects_values(
        &mut self,
        status_effects: &mut ActiveStatusEffects,
    ) -> Result<(), PacketError> {
        if let Some(status_effect) = &mut status_effects[StatusEffectType::IncreaseMaxHp] {
            status_effect.value = self.read_u16()? as i32;
        }

        if let Some(status_effect) = &mut status_effects[StatusEffectType::IncreaseMoveSpeed] {
            status_effect.value = self.read_u16()? as i32;
        }

        if let Some(status_effect) = &mut status_effects[StatusEffectType::DecreaseMoveSpeed] {
            status_effect.value = self.read_u16()? as i32;
        }

        if let Some(status_effect) = &mut status_effects[StatusEffectType::IncreaseAttackSpeed] {
            status_effect.value = self.read_u16()? as i32;
        }

        if let Some(status_effect) = &mut status_effects[StatusEffectType::DecreaseAttackSpeed] {
            status_effect.value = self.read_u16()? as i32;
        }

        Ok(())
    }
}

pub trait PacketWriteStatusEffects {
    fn write_status_effects_flags_u32(&mut self, status_effects: &ActiveStatusEffects);
    fn write_status_effects_values(&mut self, status_effects: &ActiveStatusEffects);
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

fn get_status_effect_value(
    status_effects: &ActiveStatusEffects,
    status_effect_type: StatusEffectType,
) -> Option<i32> {
    status_effects[status_effect_type]
        .as_ref()
        .map(|status_effect| status_effect.value)
}

impl PacketWriteStatusEffects for PacketWriter {
    fn write_status_effects_flags_u32(&mut self, status_effects: &ActiveStatusEffects) {
        let mut status_effect_flags = 0u32;

        for (status_effect_type, status_effect) in status_effects.iter() {
            if status_effect.is_some() {
                status_effect_flags |= get_status_effect_type_flag(status_effect_type);
            }
        }

        self.write_u32(status_effect_flags);
    }

    fn write_status_effects_values(&mut self, status_effects: &ActiveStatusEffects) {
        if let Some(value) =
            get_status_effect_value(status_effects, StatusEffectType::IncreaseMaxHp)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            get_status_effect_value(status_effects, StatusEffectType::IncreaseMoveSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            get_status_effect_value(status_effects, StatusEffectType::DecreaseMoveSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            get_status_effect_value(status_effects, StatusEffectType::IncreaseAttackSpeed)
        {
            self.write_u16(value as u16);
        }

        if let Some(value) =
            get_status_effect_value(status_effects, StatusEffectType::DecreaseAttackSpeed)
        {
            self.write_u16(value as u16);
        }
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketServerDamage {
    amount: B11,
    action: B5,
}

pub trait PacketReadDamage {
    fn read_damage_u16(&mut self) -> Result<(Damage, bool, bool), PacketError>;
}

impl<'a> PacketReadDamage for PacketReader<'a> {
    fn read_damage_u16(&mut self) -> Result<(Damage, bool, bool), PacketError> {
        let server_damage =
            PacketServerDamage::from_bytes(self.read_fixed_length_bytes(2)?.try_into().unwrap());
        let action = server_damage.action();

        let is_immediate = (action & 0x02) != 0;
        let apply_hit_stun = (action & 0x04) != 0;
        let is_critical = (action & 0x08) != 0;
        let is_killed = (action & 0x10) != 0;

        Ok((
            Damage {
                amount: server_damage.amount() as u32,
                is_critical,
                apply_hit_stun,
            },
            is_killed,
            is_immediate,
        ))
    }
}

pub trait PacketWriteDamage {
    fn write_damage_u16(&mut self, damage: &Damage, is_killed: bool, is_immediate: bool);
}

impl PacketWriteDamage for PacketWriter {
    fn write_damage_u16(&mut self, damage: &Damage, is_killed: bool, is_immediate: bool) {
        let mut action = 0u8;

        if is_immediate {
            action |= 0x02;
        }

        if damage.apply_hit_stun {
            action |= 0x04;
        }

        if damage.is_critical {
            action |= 0x08;
        }

        if is_killed {
            action |= 0x10;
        }

        let damage = PacketServerDamage::new()
            .with_amount(damage.amount.min(2047) as u16)
            .with_action(action);

        for b in damage.into_bytes().iter() {
            self.write_u8(*b);
        }
    }
}

pub trait PacketReadEntityId {
    fn read_entity_id(&mut self) -> Result<ClientEntityId, PacketError>;
    fn read_option_entity_id(&mut self) -> Result<Option<ClientEntityId>, PacketError>;
}

impl<'a> PacketReadEntityId for PacketReader<'a> {
    fn read_entity_id(&mut self) -> Result<ClientEntityId, PacketError> {
        let entity_id = self.read_u16()?;
        if entity_id == 0 {
            Err(PacketError::InvalidPacket)
        } else {
            Ok(ClientEntityId(entity_id as usize))
        }
    }

    fn read_option_entity_id(&mut self) -> Result<Option<ClientEntityId>, PacketError> {
        let entity_id = self.read_u16()?;
        if entity_id == 0 {
            Ok(None)
        } else {
            Ok(Some(ClientEntityId(entity_id as usize)))
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

pub trait PacketReadCommandState {
    fn read_command_state(&mut self) -> Result<CommandState, PacketError>;
}

impl<'a> PacketReadCommandState for PacketReader<'a> {
    fn read_command_state(&mut self) -> Result<CommandState, PacketError> {
        match self.read_u16()? {
            0 => Ok(CommandState::Stop),
            1 => Ok(CommandState::Move),
            2 => Ok(CommandState::Attack),
            3 => Ok(CommandState::Die),
            4 => Ok(CommandState::PickupItemDrop),
            6 => Ok(CommandState::CastSkillSelf),
            7 => Ok(CommandState::CastSkillTargetEntity),
            8 => Ok(CommandState::CastSkillTargetPosition),
            9 => Ok(CommandState::RunAway),
            10 => Ok(CommandState::Sit),
            11 => Ok(CommandState::PersonalStore),
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

pub trait PacketWriteCommandState {
    fn write_command_state(&mut self, command: &CommandState);
}

impl PacketWriteCommandState for PacketWriter {
    fn write_command_state(&mut self, command: &CommandState) {
        let command_id = match command {
            CommandState::Stop | CommandState::Emote => 0,
            CommandState::Move => 1,
            CommandState::Attack => 2,
            CommandState::Die => 3,
            CommandState::PickupItemDrop => 4,
            CommandState::CastSkillSelf => 6,
            CommandState::CastSkillTargetEntity => 7,
            CommandState::CastSkillTargetPosition => 8,
            CommandState::RunAway => 9,
            CommandState::Sit => 10,
            CommandState::PersonalStore => 11,
        };
        self.write_u16(command_id);
    }
}

pub trait PacketReadPartyMemberInfo {
    fn read_party_member_info(&mut self) -> Result<PartyMemberInfo, PacketError>;
    fn read_party_member_info_online(&mut self) -> Result<PartyMemberInfoOnline, PacketError>;
    fn read_party_member_info_offline(&mut self) -> Result<PartyMemberInfoOffline, PacketError>;
}

impl<'a> PacketReadPartyMemberInfo for PacketReader<'a> {
    fn read_party_member_info(&mut self) -> Result<PartyMemberInfo, PacketError> {
        let character_id = self.read_u32()?;
        let entity_id = self.read_option_entity_id()?;
        let max_health = self.read_u16()?;
        let health_points = self.read_u16()?;
        let mut status_effects = ActiveStatusEffects::default();
        self.read_status_effects_flags_u32(&mut status_effects)?;
        let concentration = self.read_u16()?;
        let health_recovery = self.read_u8()?;
        let mana_recovery = self.read_u8()?;
        let stamina = self.read_u16()?;
        let name = self.read_null_terminated_utf8()?.to_string();

        if let Some(entity_id) = entity_id {
            Ok(PartyMemberInfo::Online(PartyMemberInfoOnline {
                character_id,
                name,
                entity_id,
                health_points: HealthPoints::new(health_points as i32),
                status_effects,
                max_health: max_health as i32,
                concentration: concentration as i32,
                health_recovery: health_recovery as i32,
                mana_recovery: mana_recovery as i32,
                stamina: Stamina::new(stamina as u32),
            }))
        } else {
            Ok(PartyMemberInfo::Offline(PartyMemberInfoOffline {
                character_id,
                name,
            }))
        }
    }

    fn read_party_member_info_online(&mut self) -> Result<PartyMemberInfoOnline, PacketError> {
        match self.read_party_member_info()? {
            PartyMemberInfo::Online(online) => Ok(online),
            PartyMemberInfo::Offline(_) => Err(PacketError::InvalidPacket),
        }
    }

    fn read_party_member_info_offline(&mut self) -> Result<PartyMemberInfoOffline, PacketError> {
        match self.read_party_member_info()? {
            PartyMemberInfo::Online(_) => Err(PacketError::InvalidPacket),
            PartyMemberInfo::Offline(offline) => Ok(offline),
        }
    }
}

pub trait PacketWritePartyMemberInfo {
    fn write_party_member_info(&mut self, party_member: &PartyMemberInfo);
    fn write_party_member_info_online(&mut self, party_member: &PartyMemberInfoOnline);
    fn write_party_member_info_offline(&mut self, party_member: &PartyMemberInfoOffline);
}

impl PacketWritePartyMemberInfo for PacketWriter {
    fn write_party_member_info_online(&mut self, party_member: &PartyMemberInfoOnline) {
        self.write_u32(party_member.character_id);
        self.write_entity_id(party_member.entity_id);
        self.write_u16(party_member.max_health as u16);
        self.write_u16(party_member.health_points.hp as u16);
        self.write_status_effects_flags_u32(&party_member.status_effects);
        self.write_u16(party_member.concentration as u16);
        self.write_u8(party_member.health_recovery as u8);
        self.write_u8(party_member.mana_recovery as u8);
        self.write_u16(party_member.stamina.stamina as u16);
        self.write_null_terminated_utf8(&party_member.name);
    }

    fn write_party_member_info_offline(&mut self, party_member: &PartyMemberInfoOffline) {
        self.write_u32(party_member.character_id);
        self.write_option_entity_id(None);
        self.write_u16(0);
        self.write_u16(0);
        self.write_u32(0);
        self.write_u16(0);
        self.write_u8(0);
        self.write_u8(0);
        self.write_u16(0);
        self.write_null_terminated_utf8(&party_member.name);
    }

    fn write_party_member_info(&mut self, party_member: &PartyMemberInfo) {
        match party_member {
            PartyMemberInfo::Online(party_member) => {
                self.write_party_member_info_online(party_member);
            }
            PartyMemberInfo::Offline(party_member) => {
                self.write_party_member_info_offline(party_member);
            }
        }
    }
}

pub trait PacketReadPartyRules {
    fn read_party_rules(&mut self) -> Result<(PartyItemSharing, PartyXpSharing), PacketError>;
}

impl<'a> PacketReadPartyRules for PacketReader<'a> {
    fn read_party_rules(&mut self) -> Result<(PartyItemSharing, PartyXpSharing), PacketError> {
        let rules = self.read_u8()?;

        Ok((
            if (rules & 0x80) == 0 {
                PartyItemSharing::EqualLootDistribution
            } else {
                PartyItemSharing::AcquisitionOrder
            },
            if (rules & 0x01) == 0 {
                PartyXpSharing::EqualShare
            } else {
                PartyXpSharing::DistributedByLevel
            },
        ))
    }
}

pub trait PacketWritePartyRules {
    fn write_party_rules(&mut self, item_sharing: &PartyItemSharing, xp_sharing: &PartyXpSharing);
}

impl PacketWritePartyRules for PacketWriter {
    fn write_party_rules(&mut self, item_sharing: &PartyItemSharing, xp_sharing: &PartyXpSharing) {
        let mut bits = 0;

        if matches!(item_sharing, PartyItemSharing::AcquisitionOrder) {
            bits |= 0x80;
        }

        if matches!(xp_sharing, PartyXpSharing::DistributedByLevel) {
            bits |= 0x01;
        }

        self.write_u8(bits);
    }
}

pub trait PacketReadClanMemberPosition {
    fn read_clan_member_position_u8(&mut self) -> Result<ClanMemberPosition, PacketError>;
}

impl<'a> PacketReadClanMemberPosition for PacketReader<'a> {
    fn read_clan_member_position_u8(&mut self) -> Result<ClanMemberPosition, PacketError> {
        match self.read_u8()? {
            0 => Ok(ClanMemberPosition::Penalty),
            1 => Ok(ClanMemberPosition::Junior),
            2 => Ok(ClanMemberPosition::Senior),
            3 => Ok(ClanMemberPosition::Veteran),
            4 => Ok(ClanMemberPosition::Commander),
            5 => Ok(ClanMemberPosition::DeputyMaster),
            6 => Ok(ClanMemberPosition::Master),
            _ => Err(PacketError::InvalidPacket),
        }
    }
}

pub trait PacketWriteClanMemberPosition {
    fn write_clan_member_position_u8(&mut self, position: &ClanMemberPosition);
}

impl PacketWriteClanMemberPosition for PacketWriter {
    fn write_clan_member_position_u8(&mut self, position: &ClanMemberPosition) {
        let value = match position {
            ClanMemberPosition::Penalty => 0,
            ClanMemberPosition::Junior => 1,
            ClanMemberPosition::Senior => 2,
            ClanMemberPosition::Veteran => 3,
            ClanMemberPosition::Commander => 4,
            ClanMemberPosition::DeputyMaster => 5,
            ClanMemberPosition::Master => 6,
        };
        self.write_u8(value);
    }
}

pub trait PacketReadClanMark {
    fn read_clan_mark_u32(&mut self) -> Result<ClanMark, PacketError>;
}

impl<'a> PacketReadClanMark for PacketReader<'a> {
    fn read_clan_mark_u32(&mut self) -> Result<ClanMark, PacketError> {
        let background = self.read_u16()?;
        let foreground = self.read_u16()?;

        if background == 0 {
            Ok(ClanMark::Custom { crc16: foreground })
        } else {
            Ok(ClanMark::Premade {
                foreground,
                background,
            })
        }
    }
}

pub trait PacketWriteClanMark {
    fn write_clan_mark_u32(&mut self, mark: &ClanMark);
}

impl PacketWriteClanMark for PacketWriter {
    fn write_clan_mark_u32(&mut self, mark: &ClanMark) {
        match *mark {
            ClanMark::Premade {
                foreground,
                background,
            } => {
                self.write_u16(background);
                self.write_u16(foreground);
            }
            ClanMark::Custom { crc16 } => {
                self.write_u16(0);
                self.write_u16(crc16);
            }
        };
    }
}
