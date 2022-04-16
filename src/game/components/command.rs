use bevy::ecs::prelude::{Component, Entity};
use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use rose_data::{Item, MotionId, SkillId};
use rose_game_common::{
    components::{ItemSlot, MoveMode},
    data::Damage,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandMove {
    pub destination: Vec3,
    pub target: Option<Entity>,
    pub move_mode: Option<MoveMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandStop {
    pub send_message: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandDie {
    pub killer: Option<Entity>,
    pub damage: Option<Damage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandAttack {
    pub target: Entity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPickupItemDrop {
    pub target: Entity,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CommandCastSkillTarget {
    Entity(Entity),
    Position(Vec2),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandCastSkill {
    pub skill_id: SkillId,
    pub skill_target: Option<CommandCastSkillTarget>,
    pub use_item: Option<(ItemSlot, Item)>,
    pub cast_motion_id: Option<MotionId>,
    pub action_motion_id: Option<MotionId>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CommandSit {
    Sitting,
    Sit,
    Standing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandEmote {
    pub motion_id: MotionId,
    pub is_stop: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CommandData {
    Die(CommandDie),
    Stop(CommandStop),
    Move(CommandMove),
    Attack(CommandAttack),
    PickupItemDrop(CommandPickupItemDrop),
    PersonalStore,
    CastSkill(CommandCastSkill),
    Sit(CommandSit),
    Emote(CommandEmote),
}

impl CommandData {
    pub fn is_manual_complete(&self) -> bool {
        matches!(*self, CommandData::Sit(_) | CommandData::PersonalStore)
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

impl Command {
    pub fn new(command: CommandData, required_duration: Option<Duration>) -> Self {
        Self {
            command,
            duration: Duration::new(0, 0),
            required_duration,
        }
    }

    pub fn default() -> Self {
        Self::with_stop()
    }

    pub fn get_target(&self) -> Option<Entity> {
        match self.command {
            CommandData::Attack(CommandAttack { target, .. }) => Some(target),
            CommandData::Move(CommandMove { target, .. }) => target,
            _ => None,
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self.command, CommandData::Die(_))
    }

    pub fn is_sit(&self) -> bool {
        matches!(
            self.command,
            CommandData::Sit(CommandSit::Sit) | CommandData::Sit(CommandSit::Sitting)
        )
    }

    pub fn with_die(
        killer: Option<Entity>,
        damage: Option<Damage>,
        duration: Option<Duration>,
    ) -> Self {
        Self::new(CommandData::Die(CommandDie { killer, damage }), duration)
    }

    pub fn with_move(
        destination: Vec3,
        target: Option<Entity>,
        move_mode: Option<MoveMode>,
    ) -> Self {
        Self::new(
            CommandData::Move(CommandMove {
                destination,
                target,
                move_mode,
            }),
            None,
        )
    }

    pub fn with_attack(target: Entity, duration: Duration) -> Self {
        Self::new(
            CommandData::Attack(CommandAttack { target }),
            Some(duration),
        )
    }

    pub fn with_pickup_item_drop(target: Entity, duration: Duration) -> Self {
        Self::new(
            CommandData::PickupItemDrop(CommandPickupItemDrop { target }),
            Some(duration),
        )
    }

    pub fn with_sit() -> Self {
        Self::new(CommandData::Sit(CommandSit::Sit), None)
    }

    pub fn with_sitting(duration: Duration) -> Self {
        Self::new(CommandData::Sit(CommandSit::Sitting), Some(duration))
    }

    pub fn with_standing(duration: Duration) -> Self {
        Self::new(CommandData::Sit(CommandSit::Standing), Some(duration))
    }

    pub fn with_emote(motion_id: MotionId, is_stop: bool, duration: Duration) -> Self {
        Self::new(
            CommandData::Emote(CommandEmote { motion_id, is_stop }),
            Some(duration),
        )
    }

    pub fn with_stop() -> Self {
        Self::new(
            CommandData::Stop(CommandStop {
                send_message: false,
            }),
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
            CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target,
                use_item: None,
                cast_motion_id: None,
                action_motion_id: None,
            }),
            Some(casting_duration + action_duration),
        )
    }
}

impl From<&Command> for rose_game_common::messages::server::CommandState {
    fn from(command: &Command) -> Self {
        match command.command {
            CommandData::Die(_) => Self::Die,
            CommandData::Stop(_) => Self::Stop,
            CommandData::Move(_) => Self::Move,
            CommandData::Attack(_) => Self::Attack,
            CommandData::PickupItemDrop(_) => Self::PickupItemDrop,
            CommandData::PersonalStore => Self::PersonalStore,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: None, ..
            }) => Self::CastSkillSelf,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: Some(CommandCastSkillTarget::Entity(_)),
                ..
            }) => Self::CastSkillTargetEntity,
            CommandData::CastSkill(CommandCastSkill {
                skill_target: Some(CommandCastSkillTarget::Position(_)),
                ..
            }) => Self::CastSkillTargetPosition,
            CommandData::Sit(_) => Self::Sit,
            CommandData::Emote(_) => Self::Emote,
        }
    }
}
