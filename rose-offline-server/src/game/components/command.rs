use bevy::ecs::prelude::{Component, Entity};
use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use rose_data::{Item, MotionId, SkillId};
use rose_game_common::{
    components::{ItemSlot, MoveMode},
    data::Damage,
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CommandCastSkillTarget {
    Entity(Entity),
    Position(Vec2),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandData {
    Die {
        /// The entity which killed us
        killer: Option<Entity>,

        /// The damage which killed us
        damage: Option<Damage>,
    },
    Stop {
        /// Whether to send a network message when this command starts.
        send_message: bool,
    },
    Move {
        destination: Vec3,
        target: Option<Entity>,
        move_mode: Option<MoveMode>,
    },
    Attack {
        target: Entity,
    },
    PickupItemDrop {
        target: Entity,
    },
    PersonalStore,
    CastSkill {
        skill_id: SkillId,
        skill_target: Option<CommandCastSkillTarget>,
        use_item: Option<(ItemSlot, Item)>,
        cast_motion_id: Option<MotionId>,
        action_motion_id: Option<MotionId>,
    },

    /// Transition to Sit
    Sitting,

    /// Sitting
    Sit,

    /// Transition away from Sit
    Standing,

    Emote {
        motion_id: MotionId,
        is_stop: bool,
    },
}

impl CommandData {
    pub fn is_manual_complete(&self) -> bool {
        matches!(
            *self,
            CommandData::Sit | CommandData::PersonalStore | CommandData::Stop { .. }
        )
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    // Current command that is executing
    pub command: CommandData,

    // How long the current command has been executing
    pub duration: Duration,

    // The duration required to complete this command, if None then the command is immediately interruptible
    pub required_duration: Option<Duration>,
}

impl Default for Command {
    fn default() -> Self {
        Self::with_stop()
    }
}

impl Command {
    pub fn new(command: CommandData, required_duration: Option<Duration>) -> Self {
        Self {
            command,
            duration: Duration::new(0, 0),
            required_duration,
        }
    }

    pub fn target_entity(&self) -> Option<Entity> {
        match self.command {
            CommandData::Attack { target, .. } => Some(target),
            CommandData::Move { target, .. } => target,
            CommandData::CastSkill {
                skill_target: Some(CommandCastSkillTarget::Entity(entity)),
                ..
            } => Some(entity),
            _ => None,
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.command, CommandData::Die { .. })
    }

    pub fn is_dead_for(&self, duration: Duration) -> bool {
        matches!(self.command, CommandData::Die { .. }) && self.duration > duration
    }

    pub fn is_stop(&self) -> bool {
        matches!(self.command, CommandData::Stop { .. })
    }

    pub fn is_stop_for(&self, duration: Duration) -> bool {
        self.is_stop() && self.duration > duration
    }

    pub fn is_sit(&self) -> bool {
        matches!(self.command, CommandData::Sit | CommandData::Sitting)
    }

    pub fn is_attack_target(&self, target_entity: Entity) -> bool {
        let CommandData::Attack { target } = self.command else {
            return false;
        };

        target == target_entity
    }

    pub fn can_equip_items(&self) -> bool {
        matches!(
            self.command,
            CommandData::Stop { .. }
                | CommandData::Move { .. }
                | CommandData::Sit
                | CommandData::Sitting
                | CommandData::Standing { .. }
                | CommandData::PickupItemDrop { .. }
        )
    }

    pub fn can_equip_ammo(&self) -> bool {
        !self.is_dead()
    }

    pub fn with_die(
        killer: Option<Entity>,
        damage: Option<Damage>,
        duration: Option<Duration>,
    ) -> Self {
        Self::new(CommandData::Die { killer, damage }, duration)
    }

    pub fn with_move(
        destination: Vec3,
        target: Option<Entity>,
        move_mode: Option<MoveMode>,
    ) -> Self {
        Self::new(
            CommandData::Move {
                destination,
                target,
                move_mode,
            },
            None,
        )
    }

    pub fn with_attack(target: Entity, duration: Duration) -> Self {
        Self::new(CommandData::Attack { target }, Some(duration))
    }

    pub fn with_pickup_item_drop(target: Entity, duration: Duration) -> Self {
        Self::new(CommandData::PickupItemDrop { target }, Some(duration))
    }

    pub fn with_sit() -> Self {
        Self::new(CommandData::Sit, None)
    }

    pub fn with_sitting(duration: Duration) -> Self {
        Self::new(CommandData::Sitting, Some(duration))
    }

    pub fn with_standing(duration: Duration) -> Self {
        Self::new(CommandData::Standing, Some(duration))
    }

    pub fn with_emote(motion_id: MotionId, is_stop: bool, duration: Duration) -> Self {
        Self::new(CommandData::Emote { motion_id, is_stop }, Some(duration))
    }

    pub fn with_stop() -> Self {
        Self::new(
            CommandData::Stop {
                send_message: false,
            },
            None,
        )
    }

    pub fn with_personal_store() -> Self {
        Self::new(
            CommandData::PersonalStore,
            Some(Duration::from_secs(u64::MAX)),
        )
    }

    pub fn with_cast_skill(
        skill_id: SkillId,
        skill_target: Option<CommandCastSkillTarget>,
        casting_duration: Duration,
        action_duration: Duration,
    ) -> Self {
        Self::new(
            CommandData::CastSkill {
                skill_id,
                skill_target,
                use_item: None,
                cast_motion_id: None,
                action_motion_id: None,
            },
            Some(casting_duration + action_duration),
        )
    }
}
