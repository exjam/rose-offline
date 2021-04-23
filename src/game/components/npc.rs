use crate::game::data::formats::ifo;

pub struct Npc {
    pub id: u32,
}

impl From<&ifo::Npc> for Npc {
    fn from(npc: &ifo::Npc) -> Self {
        Self {
            id: npc.object.object_id,
        }
    }
}
