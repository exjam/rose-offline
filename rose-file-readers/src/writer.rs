use bytes::{BufMut, BytesMut};
use encoding_rs::EUC_KR;

pub struct RoseFileWriter {
    pub buffer: BytesMut,
}

impl Default for RoseFileWriter {
    fn default() -> Self {
        Self {
            buffer: BytesMut::with_capacity(1024 * 1024),
        }
    }
}

#[allow(dead_code)]
impl RoseFileWriter {
    pub fn write_padding(&mut self, size: u64) {
        for _ in 0..size {
            self.buffer.put_u8(0);
        }
    }

    pub fn write_i8(&mut self, value: i8) {
        self.buffer.put_i8(value);
    }

    pub fn write_i16(&mut self, value: i16) {
        self.buffer.put_i16_le(value);
    }

    pub fn write_i32(&mut self, value: i32) {
        self.buffer.put_i32_le(value);
    }

    pub fn write_i64(&mut self, value: i64) {
        self.buffer.put_i64_le(value);
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer.put_u8(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.buffer.put_u16_le(value);
    }

    pub fn write_u32(&mut self, value: u32) {
        self.buffer.put_u32_le(value);
    }

    pub fn write_u64(&mut self, value: u64) {
        self.buffer.put_u64_le(value);
    }

    pub fn write_f32(&mut self, value: f32) {
        self.buffer.put_f32_le(value);
    }

    pub fn write_f64(&mut self, value: f64) {
        self.buffer.put_f64_le(value);
    }

    pub fn write_u16_length_bytes(&mut self, bytes: &[u8]) {
        self.write_u16(bytes.len() as u16);
        self.buffer.put(bytes);
    }

    pub fn write_u16_length_string(&mut self, string: &str) {
        let (encoded, _, _) = EUC_KR.encode(string);
        self.write_u16_length_bytes(&encoded);
    }
}
