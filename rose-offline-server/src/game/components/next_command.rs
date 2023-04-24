use bevy::ecs::prelude::{Component, Entity};
use bevy::math::{Vec2, Vec3};

use rose_data::{Item, MotionId, SkillId};

use crate::game::components::{CommandCastSkillTarget, CommandData, ItemSlot, MoveMode};

#[derive(Component, Default)]
pub struct NextCommand {
    pub command: Option<CommandData>,
    pub has_sent_server_message: bool,
}

impl NextCommand {
    pub fn with_command_skip_server_message(command: CommandData) -> Self {
        Self {
            command: Some(command),
            has_sent_server_message: true,
        }
    }

    pub fn with_move(
        destination: Vec3,
        target: Option<Entity>,
        move_mode: Option<MoveMode>,
    ) -> Self {
        Self {
            command: Some(CommandData::Move {
                destination,
                target,
                move_mode,
            }),
            has_sent_server_message: false,
        }
    }

    pub fn with_attack(target: Entity) -> Self {
        Self {
            command: Some(CommandData::Attack { target }),
            has_sent_server_message: false,
        }
    }

    pub fn with_emote(motion_id: MotionId, is_stop: bool) -> Self {
        Self {
            command: Some(CommandData::Emote { motion_id, is_stop }),
            has_sent_server_message: false,
        }
    }

    pub fn with_pickup_item_drop(target: Entity) -> Self {
        Self {
            command: Some(CommandData::PickupItemDrop { target }),
            has_sent_server_message: false,
        }
    }

    pub fn with_sitting() -> Self {
        Self {
            command: Some(CommandData::Sitting),
            has_sent_server_message: false,
        }
    }

    pub fn with_standing() -> Self {
        Self {
            command: Some(CommandData::Standing),
            has_sent_server_message: false,
        }
    }

    pub fn with_stop(send_message: bool) -> Self {
        Self {
            command: Some(CommandData::Stop { send_message }),
            has_sent_server_message: false,
        }
    }

    pub fn with_personal_store() -> Self {
        Self {
            command: Some(CommandData::PersonalStore),
            has_sent_server_message: false,
        }
    }

    pub fn with_cast_skill_target_self(
        skill_id: SkillId,
        use_item: Option<(ItemSlot, Item)>,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill {
                skill_id,
                skill_target: None,
                use_item,
                cast_motion_id: None,
                action_motion_id: None,
            }),
            has_sent_server_message: false,
        }
    }

    pub fn with_cast_skill_target_entity(
        skill_id: SkillId,
        target_entity: Entity,
        use_item: Option<(ItemSlot, Item)>,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Entity(target_entity)),
                use_item,
                cast_motion_id: None,
                action_motion_id: None,
            }),
            has_sent_server_message: false,
        }
    }

    pub fn with_cast_skill_target_position(skill_id: SkillId, position: Vec2) -> Self {
        Self {
            command: Some(CommandData::CastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Position(position)),
                use_item: None,
                cast_motion_id: None,
                action_motion_id: None,
            }),
            has_sent_server_message: false,
        }
    }

    pub fn with_npc_cast_skill_target(
        skill_id: SkillId,
        target_entity: Entity,
        cast_motion_id: MotionId,
        action_motion_id: MotionId,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Entity(target_entity)),
                use_item: None,
                cast_motion_id: Some(cast_motion_id),
                action_motion_id: Some(action_motion_id),
            }),
            has_sent_server_message: false,
        }
    }

    pub fn with_npc_cast_skill_self(
        skill_id: SkillId,
        cast_motion_id: MotionId,
        action_motion_id: MotionId,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill {
                skill_id,
                skill_target: None,
                use_item: None,
                cast_motion_id: Some(cast_motion_id),
                action_motion_id: Some(action_motion_id),
            }),
            has_sent_server_message: false,
        }
    }
}
