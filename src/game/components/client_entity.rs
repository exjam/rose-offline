use nalgebra::Point2;

#[derive(Clone, Debug)]
pub enum ClientEntityType {
    Character,
    Monster,
    Npc,
    DroppedItem,
}

#[derive(Clone, Copy, Debug)]
pub struct ClientEntityId(pub usize);

#[derive(Clone, Debug)]
pub struct ClientEntity {
    pub id: ClientEntityId,
    pub zone: u16,
    pub sector: Point2<u32>,
    pub entity_type: ClientEntityType,
}

impl ClientEntity {
    pub fn new(
        entity_type: ClientEntityType,
        id: ClientEntityId,
        zone: u16,
        sector: Point2<u32>,
    ) -> Self {
        Self {
            id,
            zone,
            sector,
            entity_type,
        }
    }
}
