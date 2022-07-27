use bytes::{Buf, BytesMut};

pub struct ClientPacketCodec {}

impl rose_network_common::PacketCodec for ClientPacketCodec {
    fn get_seed(&self) -> u32 {
        0
    }

    fn encrypt_packet(&self, buffer: &mut BytesMut) {
        let size = (&buffer[0..2]).get_u16_le() as usize;
        for i in 2..size {
            buffer[i] ^= b'a';
        }
    }

    fn decrypt_packet_header(&self, buffer: &mut BytesMut) -> usize {
        (&buffer[0..2]).get_u16_le() as usize
    }

    fn decrypt_packet_body(&self, _buffer: &mut BytesMut) -> bool {
        true
    }
}

pub struct ServerPacketCodec {}

impl rose_network_common::PacketCodec for ServerPacketCodec {
    fn get_seed(&self) -> u32 {
        0
    }

    fn encrypt_packet(&self, _buffer: &mut BytesMut) {}

    fn decrypt_packet_header(&self, buffer: &mut BytesMut) -> usize {
        let size = (&buffer[0..2]).get_u16_le();
        for i in 2..6 {
            buffer[i] ^= b'a';
        }
        size as usize
    }

    fn decrypt_packet_body(&self, buffer: &mut BytesMut) -> bool {
        let size = (&buffer[0..2]).get_u16_le() as usize;
        for i in 6..size {
            buffer[i] ^= b'a';
        }

        true
    }
}
