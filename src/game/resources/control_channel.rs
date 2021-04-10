use crate::game::messages::control::ControlMessage;
use crossbeam_channel::Receiver;

pub struct ControlChannel {
    pub control_rx: Receiver<ControlMessage>,
}
