use std::time::Duration;

use bevy_ecs::prelude::{Component, Entity};
use nalgebra::{Point2, Point3};

use crate::{
    data::{item::Item, Damage, MotionId, SkillId},
    game::components::{ItemSlot, MoveMode},
};

#[derive(Clone)]
pub struct CommandMove {
    pub destination: Point3<f32>,
    pub target: Option<Entity>,
    pub move_mode: Option<MoveMode>,
}

#[derive(Clone)]
pub struct CommandStop {
    pub send_message: bool,
}

#[derive(Clone)]
pub struct CommandDie {
    pub killer: Option<Entity>,
    pub damage: Option<Damage>,
}

#[derive(Clone)]
pub struct CommandAttack {
    pub target: Entity,
}

#[derive(Clone)]
pub struct CommandPickupItemDrop {
    pub target: Entity,
}

#[derive(Copy, Clone)]
pub enum CommandCastSkillTarget {
    Entity(Entity),
    Position(Point2<f32>),
}

#[derive(Clone)]
pub struct CommandCastSkill {
    pub skill_id: SkillId,
    pub skill_target: Option<CommandCastSkillTarget>,
    pub use_item: Option<(ItemSlot, Item)>,
    pub cast_motion_id: Option<MotionId>,
    pub action_motion_id: Option<MotionId>,
}

#[derive(Copy, Clone)]
pub enum CommandSit {
    Sitting,
    Sit,
    Standing,
}

#[derive(Clone)]
pub enum CommandData {
    Die(CommandDie),
    Stop(CommandStop),
    Move(CommandMove),
    Attack(CommandAttack),
    PickupItemDrop(CommandPickupItemDrop),
    PersonalStore,
    CastSkill(CommandCastSkill),
    Sit(CommandSit),
}

impl CommandData {
    pub fn is_manual_complete(&self) -> bool {
        matches!(*self, CommandData::Sit(_) | CommandData::PersonalStore)
    }
}

#[derive(Component, Clone)]
pub struct Command {
    // Current command that is executing
    pub command: CommandData,

    // How long the current command has been executing
    pub duration: Duration,

    // The duration required to complete this command, if None then the command is immediately interruptible
    pub required_duration: Option<Duration>,
}

#[derive(Component)]
pub struct NextCommand {
    pub command: Option<CommandData>,
    pub has_sent_server_message: bool,
}

impl NextCommand {
    pub fn default() -> Self {
        Self {
            command: None,
            has_sent_server_message: false,
        }
    }

    pub fn with_command_skip_server_message(command: CommandData) -> Self {
        Self {
            command: Some(command),
            has_sent_server_message: true,
        }
    }

    pub fn with_move(
        destination: Point3<f32>,
        target: Option<Entity>,
        move_mode: Option<MoveMode>,
    ) -> Self {
        Self {
            command: Some(CommandData::Move(CommandMove {
                destination,
                target,
                move_mode,
            })),
            has_sent_server_message: false,
        }
    }

    pub fn with_attack(target: Entity) -> Self {
        Self {
            command: Some(CommandData::Attack(CommandAttack { target })),
            has_sent_server_message: false,
        }
    }

    pub fn with_pickup_item_drop(target: Entity) -> Self {
        Self {
            command: Some(CommandData::PickupItemDrop(CommandPickupItemDrop {
                target,
            })),
            has_sent_server_message: false,
        }
    }

    pub fn with_sitting() -> Self {
        Self {
            command: Some(CommandData::Sit(CommandSit::Sitting)),
            has_sent_server_message: false,
        }
    }

    pub fn with_sit() -> Self {
        Self {
            command: Some(CommandData::Sit(CommandSit::Sit)),
            has_sent_server_message: false,
        }
    }

    pub fn with_standing() -> Self {
        Self {
            command: Some(CommandData::Sit(CommandSit::Standing)),
            has_sent_server_message: false,
        }
    }

    pub fn with_stop(send_message: bool) -> Self {
        Self {
            command: Some(CommandData::Stop(CommandStop { send_message })),
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
            command: Some(CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target: None,
                use_item,
                cast_motion_id: None,
                action_motion_id: None,
            })),
            has_sent_server_message: false,
        }
    }

    pub fn with_cast_skill_target_entity(
        skill_id: SkillId,
        target_entity: Entity,
        use_item: Option<(ItemSlot, Item)>,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Entity(target_entity)),
                use_item,
                cast_motion_id: None,
                action_motion_id: None,
            })),
            has_sent_server_message: false,
        }
    }

    pub fn with_cast_skill_target_position(skill_id: SkillId, position: Point2<f32>) -> Self {
        Self {
            command: Some(CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Position(position)),
                use_item: None,
                cast_motion_id: None,
                action_motion_id: None,
            })),
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
            command: Some(CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target: Some(CommandCastSkillTarget::Entity(target_entity)),
                use_item: None,
                cast_motion_id: Some(cast_motion_id),
                action_motion_id: Some(action_motion_id),
            })),
            has_sent_server_message: false,
        }
    }

    pub fn with_npc_cast_skill_self(
        skill_id: SkillId,
        cast_motion_id: MotionId,
        action_motion_id: MotionId,
    ) -> Self {
        Self {
            command: Some(CommandData::CastSkill(CommandCastSkill {
                skill_id,
                skill_target: None,
                use_item: None,
                cast_motion_id: Some(cast_motion_id),
                action_motion_id: Some(action_motion_id),
            })),
            has_sent_server_message: false,
        }
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
        destination: Point3<f32>,
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
