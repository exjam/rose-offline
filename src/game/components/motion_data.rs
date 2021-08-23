use crate::data::MotionFileData;

#[derive(Default)]
pub struct MotionDataCharacter {
    pub weapon_motion_type: usize,
    pub gender: usize,
    pub attack1: Option<MotionFileData>,
    pub attack2: Option<MotionFileData>,
    pub attack3: Option<MotionFileData>,
    pub die: Option<MotionFileData>,
    pub fall: Option<MotionFileData>,
    pub hit: Option<MotionFileData>,
    pub jump1: Option<MotionFileData>,
    pub jump2: Option<MotionFileData>,
    pub pickup_dropped_item: Option<MotionFileData>,
    pub raise: Option<MotionFileData>,
    pub run: Option<MotionFileData>,
    pub sit: Option<MotionFileData>,
    pub sitting: Option<MotionFileData>,
    pub standup: Option<MotionFileData>,
    pub stop1: Option<MotionFileData>,
    pub stop2: Option<MotionFileData>,
    pub stop3: Option<MotionFileData>,
    pub walk: Option<MotionFileData>,
}

#[derive(Default)]
pub struct MotionDataNpc {
    pub stop: Option<MotionFileData>,
    pub walk: Option<MotionFileData>,
    pub attack: Option<MotionFileData>,
    pub hit: Option<MotionFileData>,
    pub die: Option<MotionFileData>,
    pub run: Option<MotionFileData>,
    pub cast1: Option<MotionFileData>,
    pub skill_action1: Option<MotionFileData>,
    pub cast2: Option<MotionFileData>,
    pub skill_action2: Option<MotionFileData>,
    pub etc: Option<MotionFileData>,
}

pub enum MotionData {
    Character(MotionDataCharacter),
    Npc(MotionDataNpc),
}

impl MotionData {
    pub fn with_character_motions(character: MotionDataCharacter) -> Self {
        Self::Character(character)
    }

    pub fn with_npc_motions(npc: MotionDataNpc) -> Self {
        Self::Npc(npc)
    }

    pub fn get_attack(&self) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => character.attack1.as_ref(),
            MotionData::Npc(npc) => npc.attack.as_ref(),
        }
    }

    pub fn get_die(&self) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => character.die.as_ref(),
            MotionData::Npc(npc) => npc.die.as_ref(),
        }
    }

    pub fn get_pickup_dropped_item(&self) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => character.pickup_dropped_item.as_ref(),
            MotionData::Npc(_) => None,
        }
    }

    pub fn get_sit_sitting(&self) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => character.sitting.as_ref(),
            MotionData::Npc(_) => None,
        }
    }

    pub fn get_sit_standing(&self) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => character.standup.as_ref(),
            MotionData::Npc(_) => None,
        }
    }
}
