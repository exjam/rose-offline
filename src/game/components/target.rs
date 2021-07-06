use legion::Entity;

#[derive(Clone)]
pub struct Target {
    pub entity: Entity,
}

impl Target {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
