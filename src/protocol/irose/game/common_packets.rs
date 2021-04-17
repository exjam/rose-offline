use std::convert::TryInto;

use crate::{
    game::components::HotbarSlot,
    protocol::{
        packet::{PacketReader, PacketWriter},
        ProtocolError,
    },
};
use modular_bitfield::prelude::*;

#[bitfield]
#[derive(Clone, Copy)]
pub struct PacketHotbarSlot {
    slot_type: B5,
    index: B11,
}

pub fn read_hotbar_slot(reader: &mut PacketReader) -> Result<Option<HotbarSlot>, ProtocolError> {
    let slot = PacketHotbarSlot::from_bytes(reader.read_fixed_length_bytes(2)?.try_into().unwrap());
    match slot.slot_type() {
        1 => Ok(Some(HotbarSlot::Inventory(slot.index()))),
        2 => Ok(Some(HotbarSlot::Command(slot.index()))),
        3 => Ok(Some(HotbarSlot::Skill(slot.index()))),
        4 => Ok(Some(HotbarSlot::Emote(slot.index()))),
        5 => Ok(Some(HotbarSlot::Dialog(slot.index()))),
        6 => Ok(Some(HotbarSlot::ClanSkill(slot.index()))),
        _ => Ok(None),
    }
}

pub fn write_hotbar_slot(writer: &mut PacketWriter, slot: &Option<HotbarSlot>) {
    let (slot_type, index) = match slot {
        Some(HotbarSlot::Inventory(index)) => (1, *index),
        Some(HotbarSlot::Command(index)) => (2, *index),
        Some(HotbarSlot::Skill(index)) => (3, *index),
        Some(HotbarSlot::Emote(index)) => (4, *index),
        Some(HotbarSlot::Dialog(index)) => (5, *index),
        Some(HotbarSlot::ClanSkill(index)) => (6, *index),
        _ => (0, 0),
    };
    let slot = PacketHotbarSlot::new()
        .with_slot_type(slot_type)
        .with_index(index);
    writer.write_bytes(&slot.into_bytes());
}
