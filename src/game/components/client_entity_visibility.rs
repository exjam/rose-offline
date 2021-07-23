use crate::game::resources::ClientEntitySet;

#[derive(Default)]
pub struct ClientEntityVisibility {
    pub entities: ClientEntitySet,
}

impl ClientEntityVisibility {
    pub fn new() -> Self {
        Default::default()
    }
}
