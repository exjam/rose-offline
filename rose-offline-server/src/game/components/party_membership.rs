use arrayvec::ArrayVec;
use bevy::prelude::{Component, Entity};

#[derive(Component, Clone, Default)]
pub struct PartyMembership {
    pub party: Option<Entity>,
    pub pending_invites: ArrayVec<Entity, 5>,
}

impl PartyMembership {
    pub fn new(party_entity: Entity) -> Self {
        Self {
            party: Some(party_entity),
            pending_invites: ArrayVec::default(),
        }
    }
}
