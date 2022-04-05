use anyhow::bail;
use encoding_rs::EUC_KR;

use crate::{reader::RoseFileReader, RoseFile};

#[derive(Debug)]
pub enum ConMessageType {
    Close,
    NextMessage,
    ShowMessage,
    PlayerSelect,
    JumpSelect,
}

#[derive(Debug)]
pub struct ConMessage {
    pub id: u32,
    pub message_type: ConMessageType,
    pub message_value: i32,
    pub condition_function: String,
    pub action_function: String,
    pub string_id: u32,
}

#[derive(Debug)]
pub struct ConMenu {
    pub messages: Vec<ConMessage>,
}

#[derive(Debug)]
pub struct ConFile {
    pub event_functions: Vec<Option<String>>,
    pub initial_messages: Vec<ConMessage>,
    pub menus: Vec<ConMenu>,
    pub script_binary: Vec<u8>,
}

fn decode_value_u32(value: u32, key_a: u8, key_b: u8) -> u32 {
    let key = if key_a & 1 != 0 { key_a } else { key_b } as u32;
    let key_u32 = (key << 24) | (key << 16) | (key << 8) | key;
    value ^ key_u32
}

fn decode_bytes(value: &[u8], key_a: u8, key_b: u8) -> Vec<u8> {
    let key = if key_a & 1 != 0 { key_a } else { key_b };
    value.iter().map(|value| *value ^ key).collect()
}

fn decode_string(value: &[u8], key_a: u8, key_b: u8) -> String {
    let mut string_bytes = decode_bytes(value, key_a, key_b);

    // Truncate trailing 0
    if let Some(index) = string_bytes.iter().position(|x| *x == 0) {
        string_bytes.truncate(index);
    }

    // Decode EUC-KR to utf8
    match std::str::from_utf8(&string_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (decoded, _, _) = EUC_KR.decode(&string_bytes);
            decoded.into_owned()
        }
    }
}

fn decode_message_type(index: u32) -> Result<ConMessageType, anyhow::Error> {
    match index {
        0 => Ok(ConMessageType::Close),
        1 => Ok(ConMessageType::NextMessage),
        2 => Ok(ConMessageType::ShowMessage),
        3 => Ok(ConMessageType::PlayerSelect),
        4 => Ok(ConMessageType::JumpSelect),
        invalid => bail!("Invalid ConMessageType {}", invalid),
    }
}

impl RoseFile for ConFile {
    type ReadOptions = ();

    fn read(mut reader: RoseFileReader, _: &Self::ReadOptions) -> Result<Self, anyhow::Error> {
        let event_mask = reader.read_u16()?;
        let mut event_functions = Vec::with_capacity(16);
        for i in 0..16 {
            let function_name = reader.read_fixed_length_string(32)?;
            if event_mask & (1 << i) != 0 {
                event_functions.push(Some(function_name.to_string()));
            } else {
                event_functions.push(None);
            }
        }
        reader.skip(2); // padding

        let conversation_offset = reader.read_u32()? as u64;
        let script_offset = reader.read_u32()? as u64;

        let num_messages = reader.read_u32()? as usize;
        let messages_offset = conversation_offset + reader.read_u32()? as u64;

        let num_menus = reader.read_u32()? as usize;
        let menus_offset = conversation_offset + reader.read_u32()? as u64;

        let mut initial_messages = Vec::with_capacity(num_messages);
        for i in 0..num_messages {
            reader.set_position(messages_offset + i as u64 * 4);
            let message_offset = messages_offset + reader.read_u32()? as u64;
            reader.set_position(message_offset);

            let id = reader.read_u32()?;
            let message_type = decode_message_type(reader.read_u32()?)?;
            let message_value = reader.read_i32()?;
            let condition_function = reader.read_fixed_length_string(32)?.to_string();
            let action_function = reader.read_fixed_length_string(32)?.to_string();
            let string_id = reader.read_u32()?;

            initial_messages.push(ConMessage {
                id,
                message_type,
                message_value,
                condition_function,
                action_function,
                string_id,
            })
        }

        let mut menus = Vec::with_capacity(num_menus);
        for i in 0..num_menus {
            reader.set_position(menus_offset + i as u64 * 4);
            let menu_offset = menus_offset + reader.read_u32()? as u64;
            reader.set_position(menu_offset);

            let menu_size = reader.read_u32()?;
            let menu_message_count = reader.read_u32()?;

            let key_a = menu_message_count as u8;
            let key_b = menu_size as u8;

            let mut menu_messages = Vec::with_capacity(menu_message_count as usize);
            for j in 0..menu_message_count {
                reader.set_position(menu_offset + 8 + j as u64 * 4);
                let menu_message_offset =
                    menu_offset + decode_value_u32(reader.read_u32()?, key_a, key_b) as u64;
                reader.set_position(menu_message_offset);

                let id = decode_value_u32(reader.read_u32()?, key_a, key_b);
                let message_type =
                    decode_message_type(decode_value_u32(reader.read_u32()?, key_a, key_b))?;
                let message_value = decode_value_u32(reader.read_u32()?, key_a, key_b) as i32;
                let condition_function =
                    decode_string(reader.read_fixed_length_bytes(32)?, key_a, key_b);
                let action_function =
                    decode_string(reader.read_fixed_length_bytes(32)?, key_a, key_b);
                let string_id = decode_value_u32(reader.read_u32()?, key_a, key_b);

                menu_messages.push(ConMessage {
                    id,
                    message_type,
                    message_value,
                    condition_function,
                    action_function,
                    string_id,
                })
            }

            menus.push(ConMenu {
                messages: menu_messages,
            });
        }

        reader.set_position(script_offset);
        let script_size = reader.read_u32()?;
        let script_bytes = reader.read_fixed_length_bytes(script_size as usize)?;
        reader.set_position_from_end(0);
        let file_size = reader.position() as u32;
        let script_binary = decode_bytes(
            script_bytes,
            (script_size & 0xFF) as u8,
            (file_size & 0xFF) as u8,
        );

        Ok(Self {
            event_functions,
            initial_messages,
            menus,
            script_binary,
        })
    }
}
