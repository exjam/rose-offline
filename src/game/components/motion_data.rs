use bevy_ecs::prelude::Component;

use rose_data::{
    MotionCharacterAction, MotionDatabase, MotionFileData, MotionId, NpcData, NpcMotionAction,
    NpcMotionId,
};
use rose_game_common::components::CharacterGender;

#[derive(Default)]
pub struct MotionDataCharacter {
    pub weapon_motion_type: usize,
    pub gender_index: usize,
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

#[derive(Component)]
pub enum MotionData {
    Character(MotionDataCharacter),
    Npc(MotionDataNpc),
}

impl MotionData {
    pub fn from_npc(npc_data: &NpcData) -> Self {
        let get_motion = |action| {
            npc_data
                .motion_data
                .get(&NpcMotionId::new(action as u16))
                .cloned()
        };

        Self::Npc(MotionDataNpc {
            stop: get_motion(NpcMotionAction::Stop),
            walk: get_motion(NpcMotionAction::Move),
            attack: get_motion(NpcMotionAction::Attack),
            hit: get_motion(NpcMotionAction::Hit),
            die: get_motion(NpcMotionAction::Die),
            run: get_motion(NpcMotionAction::Run),
            cast1: get_motion(NpcMotionAction::Cast1),
            skill_action1: get_motion(NpcMotionAction::SkillAction1),
            cast2: get_motion(NpcMotionAction::Cast2),
            skill_action2: get_motion(NpcMotionAction::SkillAction2),
            etc: get_motion(NpcMotionAction::Etc),
        })
    }

    pub fn from_character(
        motion_database: &MotionDatabase,
        weapon_motion_type: usize,
        gender: CharacterGender,
    ) -> Self {
        let gender_index = match gender {
            CharacterGender::Male => 0,
            CharacterGender::Female => 1,
        };
        let get_motion = |action| {
            motion_database
                .get_character_motion(
                    MotionId::new(action as u16),
                    weapon_motion_type,
                    gender_index,
                )
                .cloned()
        };

        Self::Character(MotionDataCharacter {
            weapon_motion_type,
            gender_index,
            attack1: get_motion(MotionCharacterAction::Attack),
            attack2: get_motion(MotionCharacterAction::Attack2),
            attack3: get_motion(MotionCharacterAction::Attack3),
            die: get_motion(MotionCharacterAction::Die),
            fall: get_motion(MotionCharacterAction::Fall),
            hit: get_motion(MotionCharacterAction::Hit),
            jump1: get_motion(MotionCharacterAction::Jump1),
            jump2: get_motion(MotionCharacterAction::Jump2),
            pickup_dropped_item: get_motion(MotionCharacterAction::Pickitem),
            raise: get_motion(MotionCharacterAction::Raise),
            run: get_motion(MotionCharacterAction::Run),
            sit: get_motion(MotionCharacterAction::Sit),
            sitting: get_motion(MotionCharacterAction::Sitting),
            standup: get_motion(MotionCharacterAction::Standup),
            stop1: get_motion(MotionCharacterAction::Stop1),
            stop2: get_motion(MotionCharacterAction::Stop2),
            stop3: get_motion(MotionCharacterAction::Stop3),
            walk: get_motion(MotionCharacterAction::Walk),
        })
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

    pub fn get_pickup_item_drop(&self) -> Option<&MotionFileData> {
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
