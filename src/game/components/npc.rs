#[derive(Clone)]
pub struct Npc {
    pub id: u32, // TODO: NpcReference
    pub quest_index: u16,
}

impl Npc {
    pub fn new(id: u32, quest_index: u16) -> Self {
        Self { id, quest_index }
    }
}
