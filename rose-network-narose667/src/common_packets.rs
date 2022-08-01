use rose_data::{EquipmentIndex, EquipmentItem, ItemReference, ItemType};
use rose_game_common::{
    components::{CharacterGender, Equipment, MoveMode},
    messages::{server::CommandState, ClientEntityId},
};
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
            _ => Err(PacketError::InvalidPacket),
        }
    }
}
