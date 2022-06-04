use bevy::ecs::prelude::{Component, Entity};

#[derive(Component, Clone)]
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

    pub fn get_party_entity(&self) -> Option<Entity> {
        match self {
            PartyMembership::None => None,
            PartyMembership::Member(party_entity) => Some(*party_entity),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, PartyMembership::None)
    }
}
