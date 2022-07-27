use rose_game_common::components::CharacterGender;
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
