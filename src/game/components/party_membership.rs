use bevy_ecs::prelude::Entity;

#[derive(Clone)]
pub enum PartyMembership {
    None,
    Member(Entity),
}

impl PartyMembership {
    pub fn default() -> Self {
        Self::None
    }

    pub fn new(party_entity: Entity) -> Self {
        Self::Member(party_entity)
    }
}
