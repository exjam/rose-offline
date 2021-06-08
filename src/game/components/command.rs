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

pub struct Command {
    // Current command that is executing
    pub command: CommandData,

    // How long the current command has been executing
    pub duration: Duration,

    // The duration required to complete this command, if None then the command is immediately interruptible
    pub required_duration: Option<Duration>,
}

pub struct NextCommand(pub CommandData);

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
