use bytes::Buf;
use std::str;
use std::io::Cursor;

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

    pub fn read_fixed_length_utf8(&mut self, length: usize) -> Result<&'a str, ReadError> {
        match str::from_utf8(self.read_fixed_length_bytes(length)?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ReadError::UnexpectedEof),
        }
    }

    pub fn read_u16_length_utf8(&mut self) -> Result<&'a str, ReadError> {
        match str::from_utf8(self.read_u16_length_bytes()?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ReadError::UnexpectedEof),
        }
    }

    pub fn read_null_terminated_utf8(&mut self) -> Result<&'a str, ReadError> {
        match str::from_utf8(self.read_null_terminated_bytes()?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ReadError::UnexpectedEof),
        }
    }
}
