use legion::Entity;

pub struct BotListEntry {
    pub entity: Entity,
}

pub type BotList = Vec<BotListEntry>;

impl BotListEntry {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
