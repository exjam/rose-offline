use std::time::Duration;

use legion::Entity;
use nalgebra::Point3;

pub struct CommandMove {
    pub destination: Point3<f32>,
    pub target: Option<Entity>,
}

pub struct CommandAttack {
    pub target: Entity,
}

pub enum CommandData {
    Die,
    Stop,
    Move(CommandMove),
    Attack(CommandAttack),
    // TODO:
    // Pick up item
    // Die
    // Cast skill
    // Sit
}

impl CommandData {
    pub fn is_move(&self, target: Option<&Entity>, destination: &Point3<f32>) -> bool {
        if target.is_some() {
            self.is_move_to_target(target.unwrap())
        } else {
            self.is_move_to_destination(destination)
        }
    }

    pub fn is_move_to_destination(&self, position: &Point3<f32>) -> bool {
        match self {
            CommandData::Move(CommandMove { destination, .. }) => destination == position,
            _ => false,
        }
    }

    pub fn is_move_to_target(&self, entity: &Entity) -> bool {
        match self {
            CommandData::Move(CommandMove { target, .. }) => {
                target.as_ref().map_or(false, |target| target == entity)
            }
            _ => false,
        }
    }

    pub fn is_attack_target(&self, entity: &Entity) -> bool {
        match self {
            CommandData::Attack(CommandAttack { target, .. }) => target == entity,
            _ => false,
        }
    }
}

pub struct Command {
    // Current command that is executing
    pub command: CommandData,

    // How long the current command has been executing
    pub duration: Duration,

    // The duration required to complete this command, if None then the command is immediately interruptible
    pub required_duration: Option<Duration>,
}

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

    pub fn with_die() -> Self {
        Self {
            command: Some(CommandData::Die),
            has_sent_server_message: false,
        }
    }

    pub fn with_move(destination: Point3<f32>, target: Option<Entity>) -> Self {
        Self {
            command: Some(CommandData::Move(CommandMove {
                destination,
                target,
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

    pub fn with_stop() -> Self {
        Self {
            command: Some(CommandData::Stop),
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

    pub fn with_die() -> Self {
        Self::new(CommandData::Die, Some(Duration::new(u64::MAX, 0)))
    }

    pub fn with_move(destination: Point3<f32>, target: Option<Entity>) -> Self {
        Self::new(
            CommandData::Move(CommandMove {
                destination,
                target,
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

    pub fn with_stop() -> Self {
        Self::new(CommandData::Stop, None)
    }
}
