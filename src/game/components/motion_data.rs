use crate::data::{
    MotionCharacterAction, MotionDatabase, MotionFileData, MotionReference, NpcMotionAction,
};

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

    #[allow(dead_code)]
    pub fn get_motion<'a>(
        &self,
        motion_database: &'a MotionDatabase,
        motion: MotionReference,
    ) -> Option<&'a MotionFileData> {
        match self {
            MotionData::Character(character) => motion_database.get_character_motion(
                motion,
                character.weapon_motion_type,
                character.gender,
            ),
            MotionData::Npc(_) => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_character_action_motion(
        &self,
        action: MotionCharacterAction,
    ) -> Option<&MotionFileData> {
        match self {
            MotionData::Character(character) => match action {
                MotionCharacterAction::Attack => character.attack1.as_ref(),
                MotionCharacterAction::Attack2 => character.attack2.as_ref(),
                MotionCharacterAction::Attack3 => character.attack3.as_ref(),
                MotionCharacterAction::Die => character.die.as_ref(),
                MotionCharacterAction::Fall => character.fall.as_ref(),
                MotionCharacterAction::Hit => character.hit.as_ref(),
                MotionCharacterAction::Jump1 => character.jump1.as_ref(),
                MotionCharacterAction::Jump2 => character.jump2.as_ref(),
                MotionCharacterAction::Pickitem => character.pickup_dropped_item.as_ref(),
                MotionCharacterAction::Raise => character.raise.as_ref(),
                MotionCharacterAction::Run => character.run.as_ref(),
                MotionCharacterAction::Sit => character.sit.as_ref(),
                MotionCharacterAction::Sitting => character.sitting.as_ref(),
                MotionCharacterAction::Standup => character.standup.as_ref(),
                MotionCharacterAction::Stop1 => character.stop1.as_ref(),
                MotionCharacterAction::Stop2 => character.stop2.as_ref(),
                MotionCharacterAction::Stop3 => character.stop3.as_ref(),
                MotionCharacterAction::Walk => character.walk.as_ref(),
            },
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_npc_action_motion(&self, action: NpcMotionAction) -> Option<&MotionFileData> {
        match self {
            MotionData::Npc(npc) => match action {
                NpcMotionAction::Stop => npc.stop.as_ref(),
                NpcMotionAction::Move => npc.walk.as_ref(),
                NpcMotionAction::Attack => npc.attack.as_ref(),
                NpcMotionAction::Hit => npc.hit.as_ref(),
                NpcMotionAction::Die => npc.die.as_ref(),
                NpcMotionAction::Run => npc.run.as_ref(),
                NpcMotionAction::Cast1 => npc.cast1.as_ref(),
                NpcMotionAction::SkillAction1 => npc.skill_action1.as_ref(),
                NpcMotionAction::Cast2 => npc.cast2.as_ref(),
                NpcMotionAction::SkillAction2 => npc.skill_action2.as_ref(),
                NpcMotionAction::Etc => npc.etc.as_ref(),
            },
            _ => None,
        }
    }
}
