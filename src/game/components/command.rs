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
}
