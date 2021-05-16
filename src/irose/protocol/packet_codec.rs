#![allow(dead_code)]

use bytes::{Buf, BytesMut};
use modular_bitfield::prelude::*;
use std::convert::TryInto;
use std::num::Wrapping;

pub const IROSE_112_TABLE: [u8; 256] = [
    0x00, 0x5E, 0xBC, 0xE2, 0x61, 0x3F, 0xDD, 0x83, 0xC2, 0x9C, 0x7E, 0x20, 0xA3, 0xFD, 0x1F, 0x41,
    0x9D, 0xC3, 0x21, 0x7F, 0xFC, 0xA2, 0x40, 0x1E, 0x5F, 0x01, 0xE3, 0xBD, 0x3E, 0x60, 0x82, 0xDC,
    0x23, 0x7D, 0x9F, 0xC1, 0x42, 0x1C, 0xFE, 0xA0, 0xE1, 0xBF, 0x5D, 0x03, 0x80, 0xDE, 0x3C, 0x62,
    0xBE, 0xE0, 0x02, 0x5C, 0xDF, 0x81, 0x63, 0x3D, 0x7C, 0x22, 0xC0, 0x9E, 0x1D, 0x43, 0xA1, 0xFF,
    0x46, 0x18, 0xFA, 0xA4, 0x27, 0x79, 0x9B, 0xC5, 0x84, 0xDA, 0x38, 0x66, 0xE5, 0xBB, 0x59, 0x07,
    0xDB, 0x85, 0x67, 0x39, 0xBA, 0xE4, 0x06, 0x58, 0x19, 0x47, 0xA5, 0xFB, 0x78, 0x26, 0xC4, 0x9A,
    0x65, 0x3B, 0xD9, 0x87, 0x04, 0x5A, 0xB8, 0xE6, 0xA7, 0xF9, 0x1B, 0x45, 0xC6, 0x98, 0x7A, 0x24,
    0xF8, 0xA6, 0x44, 0x1A, 0x99, 0xC7, 0x25, 0x7B, 0x3A, 0x64, 0x86, 0xD8, 0x5B, 0x05, 0xE7, 0xB9,
    0x8C, 0xD2, 0x30, 0x6E, 0xED, 0xB3, 0x51, 0x0F, 0x4E, 0x10, 0xF2, 0xAC, 0x2F, 0x71, 0x93, 0xCD,
    0x11, 0x4F, 0xAD, 0xF3, 0x70, 0x2E, 0xCC, 0x92, 0xD3, 0x8D, 0x6F, 0x31, 0xB2, 0xEC, 0x0E, 0x50,
    0xAF, 0xF1, 0x13, 0x4D, 0xCE, 0x90, 0x72, 0x2C, 0x6D, 0x33, 0xD1, 0x8F, 0x0C, 0x52, 0xB0, 0xEE,
    0x32, 0x6C, 0x8E, 0xD0, 0x53, 0x0D, 0xEF, 0xB1, 0xF0, 0xAE, 0x4C, 0x12, 0x91, 0xCF, 0x2D, 0x73,
    0xCA, 0x94, 0x76, 0x28, 0xAB, 0xF5, 0x17, 0x49, 0x08, 0x56, 0xB4, 0xEA, 0x69, 0x37, 0xD5, 0x8B,
    0x57, 0x09, 0xEB, 0xB5, 0x36, 0x68, 0x8A, 0xD4, 0x95, 0xCB, 0x29, 0x77, 0xF4, 0xAA, 0x48, 0x16,
    0xE9, 0xB7, 0x55, 0x0B, 0x88, 0xD6, 0x34, 0x6A, 0x2B, 0x75, 0x97, 0xC9, 0x4A, 0x14, 0xF6, 0xA8,
    0x74, 0x2A, 0xC8, 0x96, 0x15, 0x4B, 0xA9, 0xF7, 0xB6, 0xE8, 0x0A, 0x54, 0xD7, 0x89, 0x6B, 0x35,
];

#[derive(Clone, Copy)]
enum SeedType {
    VC,
    BC,
    AC,
    MY,
}

struct Random {
    vc: Wrapping<i32>,
    bc: Wrapping<i32>,
    ac: Wrapping<i32>,
    my: Wrapping<i32>,
}

impl Random {
    pub fn new(seed: u32) -> Random {
        Random {
            vc: Wrapping(seed as i32),
            bc: Wrapping(seed as i32),
            ac: Wrapping(seed as i32),
            my: Wrapping(seed as i32),
        }
    }

    pub fn next_bc(&mut self) -> u32 {
        self.bc = Wrapping(0x8088405) * self.bc + Wrapping(1);
        return (self.bc / Wrapping(0x10000)).0 as u32;
    }

    pub fn next_ac(&mut self) -> u32 {
        self.ac = (Wrapping(0x41C64E6D) * self.ac + Wrapping(12345)) & Wrapping(0x7FFFFFFF);
        return self.ac.0 as u32;
    }

    pub fn next_my(&mut self) -> u32 {
        self.my = Wrapping(0x41C64E6D) * self.my + Wrapping(12345);
        return (self.my / Wrapping(0x10000)).0 as u32;
    }

    pub fn next_vc(&mut self) -> u32 {
        self.vc = (Wrapping(0x343FD) * self.vc + Wrapping(0x269EC3)) & Wrapping(0x7FFFFFFF);
        return (self.vc / Wrapping(0x10000)).0 as u32;
    }

    pub fn get_next_fn(seed_type: SeedType) -> fn(&mut Random) -> u32 {
        match seed_type {
            SeedType::BC => Random::next_bc,
            SeedType::AC => Random::next_ac,
            SeedType::MY => Random::next_my,
            SeedType::VC => Random::next_vc,
        }
    }
}

fn seed_table(table: &mut [u32; 16 * 2048], mut seed: &mut Random, seed_types: &Vec<SeedType>) {
    for i in 0..16 {
        let seed_next_fn = Random::get_next_fn(seed_types[i]);
        let table_start = i * 2048;
        let mut size = 0usize;

        while size < 2048 {
            let next = seed_next_fn(&mut seed);

            // Yes this really is supposed to be 0..size
            if !table[0..size].contains(&next) {
                table[table_start + size] = next;
                size += 1;
            }
        }
    }
}

fn seed_index(index: &mut [u16; 512], seed: &mut Random, seed_type: SeedType) {
    let seed_next_fn = Random::get_next_fn(seed_type);
    let mut pos = 0;

    while pos < 512 {
        let next = (seed_next_fn(seed) & 0x7ff) as u16;
        if !index[0..pos].contains(&next) {
            index[pos] = next;
            pos += 1;
        }
    }
}

fn get_seed_type(i: u32) -> SeedType {
    match i & 3 {
        0 => SeedType::BC,
        1 => SeedType::AC,
        2 => SeedType::MY,
        _ => SeedType::VC,
    }
}

#[bitfield]
#[derive(Clone, Copy)]
pub struct Head {
    add_buffer_len: B11,
    command: B11,
    add_table_value: B11,
    encrypt_add_value: B4,
    encrypt_value: B3,
}

#[bitfield]
#[derive(Clone, Copy)]
struct HeadDecrypted {
    add_buffer_len1: B3,
    add_buffer_len2: B3,
    add_buffer_len3: B3,
    add_buffer_len4: B2,
    command1: B3,
    command2: B3,
    command3: B3,
    command4: B2,
    add_table_value1: B3,
    add_table_value2: B3,
    add_table_value3: B3,
    add_table_value4: B2,
    encrypt_add_value1: B2,
    encrypt_add_value2: B2,
    encrypt_value1: B3,
}

#[bitfield]
struct HeadCryptedServer {
    add_buffer_len2: B3,
    add_table_value1: B3,
    command3: B3,
    encrypt_value1: B3,
    add_buffer_len3: B3,
    add_table_value3: B3,
    command2: B3,
    add_table_value4: B2,
    command1: B3,
    encrypt_add_value1: B2,
    add_buffer_len4: B2,
    encrypt_add_value2: B2,
    add_buffer_len1: B3,
    add_table_value2: B3,
    command4: B2,
}

#[allow(dead_code)]
#[bitfield]
#[derive(Clone, Copy)]
struct HeadCryptedClient {
    command2: B3,
    add_table_value2: B3,
    add_buffer_len1: B3,
    add_table_value3: B3,
    command1: B3,
    encrypt_value1: B3,
    add_buffer_len2: B3,
    encrypt_add_value2: B2,
    add_buffer_len3: B3,
    add_table_value4: B2,
    command4: B2,
    encrypt_add_value1: B2,
    command3: B3,
    add_table_value1: B3,
    add_buffer_len4: B2,
}

impl HeadCryptedServer {
    pub fn encode_main(self: &mut Self, head: &Head) {
        let b = HeadDecrypted::from_bytes(head.into_bytes());
        self.set_add_buffer_len1(b.add_buffer_len1());
        self.set_add_buffer_len2(b.add_buffer_len2());
        self.set_add_buffer_len3(b.add_buffer_len3());
        self.set_add_buffer_len4(b.add_buffer_len4());
        self.set_command1(b.command1());
        self.set_command2(b.command2());
        self.set_command3(b.command3());
        self.set_command4(b.command4());
        self.set_encrypt_value1(b.encrypt_value1());
        self.set_encrypt_add_value1(b.encrypt_add_value1());
        self.set_encrypt_add_value2(b.encrypt_add_value2());
    }

    fn encode_final(self: &mut Self, head: &Head) {
        let b = HeadDecrypted::from_bytes(head.into_bytes());
        self.set_add_table_value1(b.add_table_value1());
        self.set_add_table_value2(b.add_table_value2());
        self.set_add_table_value3(b.add_table_value3());
        self.set_add_table_value4(b.add_table_value4());
    }
}

impl Head {
    fn decode_client_main(self: &mut Self, b: &HeadCryptedClient) {
        let mut a = HeadDecrypted::from_bytes(self.into_bytes());
        a.set_add_buffer_len1(b.add_buffer_len1());
        a.set_add_buffer_len2(b.add_buffer_len2());
        a.set_add_buffer_len3(b.add_buffer_len3());
        a.set_add_buffer_len4(b.add_buffer_len4());
        a.set_command1(b.command1());
        a.set_command2(b.command2());
        a.set_command3(b.command3());
        a.set_command4(b.command4());
        a.set_encrypt_value1(b.encrypt_value1());
        a.set_encrypt_add_value1(b.encrypt_add_value1());
        a.set_encrypt_add_value2(b.encrypt_add_value2());
        *self = Head::from_bytes(a.into_bytes());
    }

    fn decode_client_final(self: &mut Self, b: &HeadCryptedClient) {
        let mut a = HeadDecrypted::from_bytes(self.into_bytes());
        a.set_add_table_value1(b.add_table_value1());
        a.set_add_table_value2(b.add_table_value2());
        a.set_add_table_value3(b.add_table_value3());
        a.set_add_table_value4(b.add_table_value4());
        *self = Head::from_bytes(a.into_bytes());
    }
}

pub struct PacketCodec {
    seed: u32,
    crc_table: &'static [u8; 256],
    table: [u32; 16 * 2048],
    index: [u16; 512],
}

impl PacketCodec {
    pub fn default(crc_table: &'static [u8; 256]) -> PacketCodec {
        let mut seed = Random::new(0x0042D266u32);
        let seed_types: Vec<SeedType> = (0..16).map(get_seed_type).collect();
        let mut crypt = PacketCodec {
            seed: 0,
            crc_table: crc_table,
            table: [0u32; 16 * 2048],
            index: [0u16; 512],
        };
        seed_table(&mut crypt.table, &mut seed, &seed_types);
        seed_index(&mut crypt.index, &mut seed, SeedType::AC);
        crypt
    }

    pub fn init(crc_table: &'static [u8; 256], init_seed: u32) -> PacketCodec {
        let mut seed = Random::new(Random::new(init_seed).next_my());
        let seed_types: Vec<SeedType> = (0..17)
            .map(|_| get_seed_type(seed.next_bc() & 0xFF))
            .collect();
        let mut crypt = PacketCodec {
            seed: init_seed,
            crc_table: crc_table,
            table: [0u32; 16 * 2048],
            index: [0u16; 512],
        };
        seed_table(&mut crypt.table, &mut seed, &seed_types);
        seed_index(&mut crypt.index, &mut seed, seed_types[16]);
        crypt
    }
}

impl crate::protocol::PacketCodec for PacketCodec {
    fn get_seed(&self) -> u32 {
        self.seed
    }

    fn encrypt_server(&self, buffer: &mut BytesMut) {
        let add_table_value = 1u16;
        let encrypt_add_value = 1u8;
        let size = (&buffer[0..2]).get_u16_le();
        let head = Head::new()
            .with_add_table_value(add_table_value)
            .with_encrypt_add_value(encrypt_add_value)
            .with_encrypt_value(((add_table_value + encrypt_add_value as u16) & 0xFF) as u8)
            .with_add_buffer_len(size)
            .with_command((&buffer[2..4]).get_u16_le());

        let mut head_server = HeadCryptedServer::from_bytes(buffer[0..5].try_into().unwrap());
        head_server.encode_main(&head);
        (&mut buffer[0..5]).copy_from_slice(&head_server.into_bytes());

        let mut checksum = 0u8;
        let head_bytes = head.into_bytes();
        for i in 0..5 {
            checksum = self.crc_table[(head_bytes[i] ^ checksum) as usize];
            buffer[i] ^= self.table[i * 2048 + add_table_value as usize] as u8;
        }

        for i in 6..size as usize {
            let table_start = ((encrypt_add_value as usize + i) & 0xF) * 2048;
            let table_offset = (add_table_value as usize + i) & 0x7FF;
            checksum = self.crc_table[(buffer[i] ^ checksum) as usize];
            buffer[i] ^= self.table[table_start + table_offset] as u8;
        }

        buffer[5] = checksum;

        let mut head_server = HeadCryptedServer::from_bytes(buffer[0..5].try_into().unwrap());
        head_server.encode_final(&head);
        (&mut buffer[0..5]).copy_from_slice(&head_server.into_bytes());
    }

    fn decrypt_client_header(&self, buffer: &mut BytesMut) -> usize {
        let mut head = Head::new();
        head.decode_client_final(&HeadCryptedClient::from_bytes(
            buffer[0..5].try_into().unwrap(),
        ));
        let add_table_value = head.add_table_value();

        for i in 0..5 {
            buffer[i] ^= self.table[i * 2048 + add_table_value as usize] as u8;
        }

        head.decode_client_main(&HeadCryptedClient::from_bytes(
            buffer[0..5].try_into().unwrap(),
        ));
        (&mut buffer[0..5]).copy_from_slice(&head.into_bytes());
        head.add_buffer_len() as usize
    }

    fn decrypt_client_body(&self, buffer: &mut BytesMut) -> bool {
        let head = Head::from_bytes(buffer[0..5].try_into().unwrap());
        let mut checksum: u8 = 0;
        for i in 0..5 {
            checksum = self.crc_table[(buffer[i] ^ checksum) as usize];
        }

        let add_buffer_len = head.add_buffer_len() as usize;
        let encrypt_add_value = head.encrypt_add_value() as usize;
        let add_table_value = head.add_table_value() as usize;
        let data_length = add_buffer_len - head.encrypt_value() as usize;
        for i in 6..data_length {
            let table_start = ((encrypt_add_value + i) & 0xF) * 2048;
            let table_offset = (add_table_value + i) & 0x7FF;
            buffer[i] ^= self.table[table_start + table_offset] as u8;
            checksum = self.crc_table[(buffer[i] ^ checksum) as usize];
        }

        if buffer[5] != checksum {
            return false;
        }

        (&mut buffer[0..2]).copy_from_slice(&data_length.to_le_bytes()[0..2]);
        (&mut buffer[2..4]).copy_from_slice(&head.command().to_le_bytes()[0..2]);
        true
    }
}
