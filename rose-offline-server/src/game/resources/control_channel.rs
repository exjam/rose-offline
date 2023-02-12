use bevy::prelude::Resource;
use crossbeam_channel::Receiver;

use crate::game::messages::control::ControlMessage;

#[derive(Resource)]
pub struct ControlChannel {
    pub control_rx: Receiver<ControlMessage>,
}

impl ControlChannel {
    pub fn new(control_rx: Receiver<ControlMessage>) -> Self {
        ControlChannel { control_rx }
    }
}
