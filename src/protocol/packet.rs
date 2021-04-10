use super::ProtocolError;
use crate::game::messages::{client::ClientMessage, server::ServerMessage};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::Cursor;
use std::str;

pub trait PacketCodec {
    fn decrypt_client_header(self: &Self, buffer: &mut BytesMut) -> usize;
    fn decrypt_client_body(self: &Self, buffer: &mut BytesMut) -> bool;

    fn encrypt_server(self: &Self, buffer: &mut BytesMut);
}

#[derive(Debug)]
pub struct Packet {
    pub command: u16,
    pub data: Bytes,
}

impl Packet {
    pub fn with_data(command: u16, data: BytesMut) -> Packet {
        Packet {
            command: command,
            data: data.freeze(),
        }
    }
}

pub struct PacketReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> From<&'a Packet> for PacketReader<'a> {
    fn from(packet: &'a Packet) -> Self {
        Self {
            cursor: Cursor::new(&packet.data[..]),
        }
    }
}

impl<'a> PacketReader<'a> {
    pub fn read_u8(&mut self) -> Result<u8, ProtocolError> {
        if self.cursor.remaining() < 1 {
            Err(ProtocolError::InvalidPacket)
        } else {
            Ok(self.cursor.get_u8())
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, ProtocolError> {
        if self.cursor.remaining() < 2 {
            Err(ProtocolError::InvalidPacket)
        } else {
            Ok(self.cursor.get_u16_le())
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, ProtocolError> {
        if self.cursor.remaining() < 4 {
            Err(ProtocolError::InvalidPacket)
        } else {
            Ok(self.cursor.get_u32_le())
        }
    }

    pub fn read_fixed_length_bytes(&mut self, length: usize) -> Result<&'a [u8], ProtocolError> {
        if self.cursor.remaining() < length {
            Err(ProtocolError::InvalidPacket)
        } else {
            let start = self.cursor.position() as usize;
            let end = start + length;
            self.cursor.set_position(end as u64);
            Ok(&self.cursor.get_ref()[start..end])
        }
    }

    pub fn read_u16_length_bytes(&mut self) -> Result<&'a [u8], ProtocolError> {
        let length = self.read_u16()?;
        self.read_fixed_length_bytes(length as usize)
    }

    pub fn read_null_terminated_bytes(&mut self) -> Result<&'a [u8], ProtocolError> {
        let start = self.cursor.position() as usize;
        let end = self.cursor.get_ref().as_ref().len();

        for i in start..end {
            if self.cursor.get_ref()[i] == 0 {
                self.cursor.set_position((i + 1) as u64);
                return Ok(&self.cursor.get_ref()[start..i]);
            }
        }

        Err(ProtocolError::InvalidPacket)
    }

    pub fn read_fixed_length_utf8(&mut self, length: usize) -> Result<&'a str, ProtocolError> {
        match str::from_utf8(self.read_fixed_length_bytes(length)?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ProtocolError::InvalidPacket),
        }
    }

    pub fn read_u16_length_utf8(&mut self) -> Result<&'a str, ProtocolError> {
        match str::from_utf8(self.read_u16_length_bytes()?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ProtocolError::InvalidPacket),
        }
    }

    pub fn read_null_terminated_utf8(&mut self) -> Result<&'a str, ProtocolError> {
        match str::from_utf8(self.read_null_terminated_bytes()?) {
            Ok(s) => return Ok(s.trim_end_matches(char::from(0))),
            Err(_) => return Err(ProtocolError::InvalidPacket),
        }
    }
}

pub struct PacketWriter {
    command: u16,
    data: BytesMut,
}

impl PacketWriter {
    pub fn new(command: u16) -> PacketWriter {
        PacketWriter {
            command,
            data: BytesMut::with_capacity(1024),
        }
    }

    pub fn write_bytes(&mut self, value: &[u8]) {
        self.data.put(value);
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.put_u8(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.put_u16_le(value);
    }

    pub fn write_f32(&mut self, value: f32) {
        self.data.put_f32_le(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.put_u32_le(value);
    }

    pub fn write_i64(&mut self, value: i64) {
        self.data.put_i64_le(value);
    }

    pub fn write_null_terminated_utf8(&mut self, value: &str) {
        self.data.put(value.as_bytes());
        self.data.put_u8(0);
    }
}

impl From<PacketWriter> for Packet {
    fn from(writer: PacketWriter) -> Packet {
        Packet::with_data(writer.command, writer.data)
    }
}
