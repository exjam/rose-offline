use nalgebra::Point2;

pub enum ClientEntityType {
    Character,
    Monster,
    Npc,
    // TODO: Dropped Item
}

#[derive(Clone, Copy, Debug)]
pub struct ClientEntityId(pub usize);

pub struct ClientEntity {
    pub id: ClientEntityId,
    pub sector: Point2<u32>,
    pub entity_type: ClientEntityType,
}

impl ClientEntity {
    pub fn new(entity_type: ClientEntityType, id: ClientEntityId, sector: Point2<u32>) -> Self {
        Self {
            id,
            sector,
            entity_type,
        }
    }
}
