use crate::game::resources::ClientEntityId;
use nalgebra::Point2;

pub struct ClientEntity {
    pub id: ClientEntityId,
    pub sector: Point2<u32>,
}

impl ClientEntity {
    pub fn new(id: ClientEntityId, sector: Point2<u32>) -> Self {
        Self { id, sector }
    }
}
