use bytes::Buf;
use encoding_rs::{EUC_KR, UTF_16LE};
use std::{
    borrow::Cow,
    io::{Cursor, Read, Seek, SeekFrom},
    str,
};
use thiserror::Error;

use crate::types::{Quat4, Vec2, Vec3, Vec4};

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

impl From<ReadError> for std::io::Error {
    fn from(error: ReadError) -> std::io::Error {
        match error {
            ReadError::UnexpectedEof => std::io::Error::from(std::io::ErrorKind::UnexpectedEof),
        }
    }
}

pub struct RoseFileReader<'a> {
    pub cursor: Cursor<&'a [u8]>,
    pub use_wide_strings: bool,
}

impl<'a> From<&'a Vec<u8>> for RoseFileReader<'a> {
    fn from(vec: &'a Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(vec),
            use_wide_strings: false,
        }
    }
}

impl<'a> From<&'a [u8]> for RoseFileReader<'a> {
    fn from(slice: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(slice),
            use_wide_strings: false,
        }
    }
}

fn decode_string(mut bytes: &[u8], use_wide_strings: bool) -> Cow<'_, str> {
    if bytes.is_empty() {
        return Cow::default();
    }

    if use_wide_strings {
        // Some fixed length strings include a null terminator, so we should trim it.
        for i in (0..(bytes.len() - 1)).step_by(2) {
            if bytes[i] == 0 && bytes[i + 1] == 0 {
                bytes = &bytes[0..i];
                break;
            }
        }

        // Decode utf16le to utf8
        let (decoded, _, _) = UTF_16LE.decode(bytes);
        decoded
    } else {
        // Some fixed length strings include a null terminator, so we should trim it.
        for (i, c) in bytes.iter().enumerate() {
            if *c == 0 {
                bytes = &bytes[0..i];
                break;
            }
        }

        // Decode EUC-KR to utf8
        match str::from_utf8(bytes) {
            Ok(s) => Cow::from(s),
            Err(_) => {
                let (decoded, _, _) = EUC_KR.decode(bytes);
                decoded
            }
        }
    }
}

#[allow(dead_code)]
impl<'a> RoseFileReader<'a> {
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

    pub fn read_i8(&mut self) -> Result<i8, ReadError> {
        if self.cursor.remaining() < 1 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_i8())
        }
    }

    pub fn read_i16(&mut self) -> Result<i16, ReadError> {
        if self.cursor.remaining() < 2 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_i16_le())
        }
    }

    pub fn read_i32(&mut self) -> Result<i32, ReadError> {
        if self.cursor.remaining() < 4 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_i32_le())
        }
    }

    pub fn read_f32(&mut self) -> Result<f32, ReadError> {
        if self.cursor.remaining() < 4 {
            Err(ReadError::UnexpectedEof)
        } else {
            Ok(self.cursor.get_f32_le())
        }
    }

    #[allow(clippy::uninit_vec)]
    pub fn read_vec<T>(&mut self, elements: usize) -> Result<Vec<T>, ReadError> {
        let bytes_length = std::mem::size_of::<T>() * elements;
        if self.cursor.remaining() < bytes_length {
            Err(ReadError::UnexpectedEof)
        } else {
            let mut result: Vec<T> = Vec::with_capacity(elements);
            unsafe {
                result.set_len(elements);
                self.cursor
                    .read_exact(std::slice::from_raw_parts_mut(
                        result.as_mut_ptr() as *mut u8,
                        bytes_length,
                    ))
                    .map_err(|_| ReadError::UnexpectedEof)?
            }
            Ok(result)
        }
    }

    pub fn read_vector2_f32(&mut self) -> Result<Vec2<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        Ok(Vec2 { x, y })
    }

    pub fn read_vector3_f32(&mut self) -> Result<Vec3<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok(Vec3 { x, y, z })
    }

    pub fn read_vector4_f32(&mut self) -> Result<Vec4<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        let w = self.read_f32()?;
        Ok(Vec4 { x, y, z, w })
    }

    pub fn read_vector4_u16(&mut self) -> Result<Vec4<u16>, ReadError> {
        let x = self.read_u16()?;
        let y = self.read_u16()?;
        let z = self.read_u16()?;
        let w = self.read_u16()?;
        Ok(Vec4 { x, y, z, w })
    }

    pub fn read_vector4_u32(&mut self) -> Result<Vec4<u32>, ReadError> {
        let x = self.read_u32()?;
        let y = self.read_u32()?;
        let z = self.read_u32()?;
        let w = self.read_u32()?;
        Ok(Vec4 { x, y, z, w })
    }

    pub fn read_quat4_xyzw_f32(&mut self) -> Result<Quat4<f32>, ReadError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        let w = self.read_f32()?;
        Ok(Quat4 { x, y, z, w })
    }

    pub fn read_quat4_wxyz_f32(&mut self) -> Result<Quat4<f32>, ReadError> {
        let w = self.read_f32()?;
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok(Quat4 { x, y, z, w })
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

    pub fn read_u32_length_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let length = self.read_u32()?;
        self.read_fixed_length_bytes(length as usize)
    }

    pub fn read_null_terminated_bytes(&mut self) -> Result<&'a [u8], ReadError> {
        let start = self.cursor.position() as usize;
        let end = self.cursor.get_ref().len() - 1;

        for i in start..end {
            if self.cursor.get_ref()[i] == 0 {
                self.cursor.set_position((i + 1) as u64);
                return Ok(&self.cursor.get_ref()[start..i]);
            }
        }

        Err(ReadError::UnexpectedEof)
    }

    pub fn read_fixed_length_string(&mut self, length: usize) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_fixed_length_bytes(length)?,
            self.use_wide_strings,
        ))
    }

    pub fn read_variable_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_variable_length_bytes()?,
            self.use_wide_strings,
        ))
    }

    pub fn read_u8_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_u8_length_bytes()?,
            self.use_wide_strings,
        ))
    }

    pub fn read_u16_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_u16_length_bytes()?,
            self.use_wide_strings,
        ))
    }

    pub fn read_u32_length_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_u32_length_bytes()?,
            self.use_wide_strings,
        ))
    }

    pub fn read_null_terminated_string(&mut self) -> Result<Cow<'a, str>, ReadError> {
        Ok(decode_string(
            self.read_null_terminated_bytes()?,
            self.use_wide_strings,
        ))
    }
}
