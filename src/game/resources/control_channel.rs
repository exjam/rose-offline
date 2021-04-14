use crate::game::messages::control::ControlMessage;
use crossbeam_channel::Receiver;

pub struct ControlChannel {
    pub control_rx: Receiver<ControlMessage>,
}

impl ControlChannel {
    pub fn new(control_rx: Receiver<ControlMessage>) -> Self {
        ControlChannel { control_rx }
    }
}
