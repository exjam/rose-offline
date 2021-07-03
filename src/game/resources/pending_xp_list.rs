use legion::Entity;

pub struct PendingXp {
    pub entity: Entity,
    pub xp: i32,
    pub source: Option<Entity>,
}

pub type PendingXpList = Vec<PendingXp>;

impl PendingXp {
    pub fn new(entity: Entity, xp: i32, source: Option<Entity>) -> Self {
        Self { entity, xp, source }
    }
}
