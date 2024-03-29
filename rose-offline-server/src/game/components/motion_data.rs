use bevy::ecs::prelude::Component;

use rose_data::{
    CharacterMotionAction, CharacterMotionDatabase, MotionFileData, NpcDatabase, NpcId,
    NpcMotionAction, VehicleMotionAction,
};
use rose_game_common::components::CharacterGender;

pub struct MotionDataCharacter {
    pub weapon_motion_type: usize,
    pub gender: CharacterGender,
    pub base_vehicle_motion_index: Option<usize>,
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

pub struct MotionDataNpc {
    pub npc_id: NpcId,
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
    pub fn from_npc(npc_database: &NpcDatabase, npc_id: NpcId) -> Self {
        let get_motion = |action| npc_database.get_npc_action_motion(npc_id, action).cloned();

        Self::Npc(MotionDataNpc {
            npc_id,
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
        character_motion_database: &CharacterMotionDatabase,
        weapon_motion_type: usize,
        gender: CharacterGender,
    ) -> Self {
        let gender_index = match gender {
            CharacterGender::Male => 0,
            CharacterGender::Female => 1,
        };
        let get_motion = |action| {
            character_motion_database
                .get_character_action_motion(action, weapon_motion_type, gender_index)
                .cloned()
        };

        Self::Character(MotionDataCharacter {
            weapon_motion_type,
            gender,
            base_vehicle_motion_index: None,
            attack1: get_motion(CharacterMotionAction::Attack),
            attack2: get_motion(CharacterMotionAction::Attack2),
            attack3: get_motion(CharacterMotionAction::Attack3),
            die: get_motion(CharacterMotionAction::Die),
            fall: get_motion(CharacterMotionAction::Fall),
            hit: get_motion(CharacterMotionAction::Hit),
            jump1: get_motion(CharacterMotionAction::Jump1),
            jump2: get_motion(CharacterMotionAction::Jump2),
            pickup_dropped_item: get_motion(CharacterMotionAction::Pickitem),
            raise: get_motion(CharacterMotionAction::Raise),
            run: get_motion(CharacterMotionAction::Run),
            sit: get_motion(CharacterMotionAction::Sit),
            sitting: get_motion(CharacterMotionAction::Sitting),
            standup: get_motion(CharacterMotionAction::Standup),
            stop1: get_motion(CharacterMotionAction::Stop1),
            stop2: get_motion(CharacterMotionAction::Stop2),
            stop3: get_motion(CharacterMotionAction::Stop3),
            walk: get_motion(CharacterMotionAction::Walk),
        })
    }

    pub fn from_vehicle(
        character_motion_database: &CharacterMotionDatabase,
        base_vehicle_motion_index: usize,
        weapon_motion_type: usize,
    ) -> Self {
        let get_motion = |action| {
            character_motion_database
                .get_vehicle_action_motion(action, base_vehicle_motion_index, weapon_motion_type)
                .cloned()
        };

        Self::Character(MotionDataCharacter {
            weapon_motion_type,
            gender: CharacterGender::Male,
            base_vehicle_motion_index: Some(base_vehicle_motion_index),
            attack1: get_motion(VehicleMotionAction::Attack1),
            attack2: get_motion(VehicleMotionAction::Attack2),
            attack3: get_motion(VehicleMotionAction::Attack3),
            die: get_motion(VehicleMotionAction::Die),
            fall: get_motion(VehicleMotionAction::Stop),
            hit: get_motion(VehicleMotionAction::Stop),
            jump1: get_motion(VehicleMotionAction::Stop),
            jump2: get_motion(VehicleMotionAction::Stop),
            pickup_dropped_item: get_motion(VehicleMotionAction::Stop),
            raise: get_motion(VehicleMotionAction::Stop),
            run: get_motion(VehicleMotionAction::Move),
            sit: get_motion(VehicleMotionAction::Stop),
            sitting: get_motion(VehicleMotionAction::Stop),
            standup: get_motion(VehicleMotionAction::Stop),
            stop1: get_motion(VehicleMotionAction::Stop),
            stop2: get_motion(VehicleMotionAction::Stop),
            stop3: get_motion(VehicleMotionAction::Stop),
            walk: get_motion(VehicleMotionAction::Move),
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
