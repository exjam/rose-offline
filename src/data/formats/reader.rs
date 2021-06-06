use bytes::Buf;
use encoding_rs::EUC_KR;
use nalgebra::{Quaternion, Unit, UnitQuaternion, Vector3};
use std::io::Seek;
use std::io::SeekFrom;
use std::str;
use std::{borrow::Cow, io::Cursor};

pub enum ReadError {
    UnexpectedEof,
}

impl From<ReadError> for std::io::Error {
    fn from(error: ReadError) -> std::io::Error {
        match error {
            ReadError::UnexpectedEof => std::io::Error::from(std::io::ErrorKind::UnexpectedEof),
        }
    }
}

pub struct FileReader<'a> {
    pub cursor: Cursor<&'a [u8]>,
}

impl<'a> From<&'a Vec<u8>> for FileReader<'a> {
    fn from(vec: &'a Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(vec),
        }
    }
}

impl<'a> From<&'a [u8]> for FileReader<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(slice),
        }
    }
}

fn decode_string(mut bytes: &[u8]) -> Cow<'_, str> {
    // Some fixed length strings include a null terminator, so we should trim it.
    for (i, c) in bytes.iter().enumerate() {
        if *c == 0 {
            bytes = &bytes[0..i];
            break;
        }
    }

    match str::from_utf8(bytes) {
        Ok(s) => Cow::from(s),
        Err(_) => {
            let (decoded, _, _) = EUC_KR.decode(bytes);
            decoded
        }
    }
}

#[allow(dead_code)]
impl<'a> FileReader<'a> {
    pub fn remaining(&self) -> usize {
        self.cursor.remaining()
    }

    pub fn skip(&mut self, distance: u64) {
        self.cursor.set_position(self.cursor.position() + distance);
    }

    pub fn position(&self) -> u64 {
        self.cursor.position()
    }

    pub fn set_position(&mut self, pos: u64) {
        self.cursor.set_position(pos);
    }

    pub fn set_position_from_end(&mut self, offset: i64) {
        self.cursor.seek(SeekFrom::End(offset)).ok();
    }

    pub fn read_u8(&mut self) -> Result<u8, ReadError> {
        if self.cursor.remaining() < 1 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_u8())
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, ReadError> {
        if self.cursor.remaining() < 2 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_u16_le())
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, ReadError> {
        if self.cursor.remaining() < 4 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_u32_le())
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, ReadError> {
        if self.cursor.remaining() < 4 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_f32_le())
        }
    }

    pub fn read_vector3_f32(&mut self) -> Result<Vector3<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok(Vector3::new(x, y, z))
    }

    pub fn read_quaternion_f32(&mut self) -> Result<UnitQuaternion<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        let w = self.read_f32()?;
        Ok(Unit::new_unchecked(Quaternion::new(w, x, y, z)))
    }

    pub fn read_fixed_length_bytes(&mut self, length: usize) -> Result<&'a [u8], ReadError> {
        if self.cursor.remaining() < length {
            Err(ReadError::UnexpectedEof)
        } else {
            let start = self.cursor.position() as usize;
            let end = start + length;
            self.cursor.set_position(end as u64);
            Ok(&self.cursor.get_ref()[start..end])
        }
    }

    pub fn read_variable_length_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let mut length = 0usize;
        loop {
            let byte = self.read_u8()?;
            length += (byte & 0x7f) as usize;
            if (byte & 0x80) == 0 {
                break;
            }
        }
        self.read_fixed_length_bytes(length as usize)
    }

    pub fn read_u8_length_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let length = self.read_u8()?;
        self.read_fixed_length_bytes(length as usize)
    }

    pub fn read_u16_length_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let length = self.read_u16()?;
        self.read_fixed_length_bytes(length as usize)
    }

    pub fn read_null_terminated_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let start = self.cursor.position() as usize;
        let end = self.cursor.get_ref().len() - 1;

        for i in start..end {
            if self.cursor.get_ref().as_ref()[i] == 0 {
                self.cursor.set_position((i + 1) as u64);
                return Ok(&self.cursor.get_ref()[start..i]);
            }
        }

        Err(ReadError::UnexpectedEof)
    }

    pub fn read_fixed_length_string(&mut self, length: usize) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(self.read_fixed_length_bytes(length)?))
    }

    pub fn read_variable_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(self.read_variable_length_bytes()?))
    }

    pub fn read_u8_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(self.read_u8_length_bytes()?))
    }

    pub fn read_u16_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(self.read_u16_length_bytes()?))
    }

    pub fn read_null_terminated_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(self.read_null_terminated_bytes()?))
    }
}
