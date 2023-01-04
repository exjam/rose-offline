use bevy::prelude::{Component, Deref, DerefMut, Entity};

#[derive(Component, Clone, Default, Deref, DerefMut)]
pub struct PartyMembership(pub Option<Entity>);

impl PartyMembership {
    pub fn new(party_entity: Entity) -> Self {
        Self(Some(party_entity))
    }

    pub fn party(&self) -> Option<Entity> {
        self.0
    }
}
