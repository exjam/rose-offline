use crossbeam_channel::Receiver;
use crate::game::messages::control::ControlMessage;

pub struct ControlClient {
    pub control_rx: Receiver<ControlMessage>,
}
