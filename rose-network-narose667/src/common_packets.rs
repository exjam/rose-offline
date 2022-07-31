use rose_data::{EquipmentIndex, EquipmentItem, ItemReference, ItemType};
use rose_game_common::components::{CharacterGender, Equipment};
use rose_network_common::{PacketError, PacketReader, PacketWriter};

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

pub trait PacketReadItems {
    fn read_equipment_item_part(
        &mut self,
        item_type: ItemType,
    ) -> Result<Option<EquipmentItem>, PacketError>;
    fn read_equipment_visible_part(&mut self) -> Result<Equipment, PacketError>;
}

impl<'a> PacketReadItems for PacketReader<'a> {
    fn read_equipment_item_part(
        &mut self,
        item_type: ItemType,
    ) -> Result<Option<EquipmentItem>, PacketError> {
        let item_number = self.read_u32()? as usize;
        let _gem_option1 = self.read_u16()?;
        let _gem_option2 = self.read_u16()?;
        let _gem_option3 = self.read_u16()?;
        let _socket_count = self.read_u8()?;
        let grade = self.read_u16()?;
        let _item_color = self.read_u32()?;

        if let Some(mut item) = EquipmentItem::new(ItemReference::new(item_type, item_number), 0) {
            item.grade = grade as u8;
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
}
