use modular_bitfield::prelude::*;
use num_traits::FromPrimitive;
use std::convert::TryInto;

use crate::{
    data::{
        item::{EquipmentItem, Item, StackableItem},
        ItemReference,
    },
    game::components::{
        Equipment, EquipmentIndex, HotbarSlot, InventoryPageType, ItemSlot, Money,
        INVENTORY_PAGE_SIZE,
    },
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

        if let Some(item_type) = FromPrimitive::from_u8(item_header.item_type()) {
            let item_reference = ItemReference::new(item_type, item_header.item_number() as usize);

            if item_type.is_stackable() {
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

pub trait PacketReadItemSlot {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, ProtocolError>;
    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, ProtocolError>;
}

fn parse_item_slot(index: usize) -> Result<ItemSlot, ProtocolError>
{
    if index == 0 {
        Err(ProtocolError::InvalidPacket)
    } else if index < 12 {
        if let Some(equipment_index) = FromPrimitive::from_usize(index) {
            Ok(ItemSlot::Equipped(equipment_index))
        } else {
            Err(ProtocolError::InvalidPacket)
        }
    } else {
        let index = index - 12;
        let page = index / INVENTORY_PAGE_SIZE;
        let slot = index % INVENTORY_PAGE_SIZE;
        match page {
            0 => Ok(ItemSlot::Inventory(InventoryPageType::Equipment, slot)),
            1 => Ok(ItemSlot::Inventory(InventoryPageType::Consumables, slot)),
            2 => Ok(ItemSlot::Inventory(InventoryPageType::Materials, slot)),
            3 => Ok(ItemSlot::Inventory(InventoryPageType::Vehicles, slot)),
            _ => Err(ProtocolError::InvalidPacket),
        }
    }
}

impl<'a> PacketReadItemSlot for PacketReader<'a> {
    fn read_item_slot_u8(&mut self) -> Result<ItemSlot, ProtocolError> {
        parse_item_slot(self.read_u8()? as usize)
    }

    fn read_item_slot_u16(&mut self) -> Result<ItemSlot, ProtocolError> {
        parse_item_slot(self.read_u16()? as usize)
    }
}
