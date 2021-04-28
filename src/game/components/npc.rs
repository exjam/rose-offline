use crate::game::data::formats::ifo;

#[derive(Clone)]
pub struct Npc {
    pub id: u32,
    pub quest_index: u16,
    pub direction: f32,
}

impl From<&ifo::Npc> for Npc {
    fn from(npc: &ifo::Npc) -> Self {
        let direction = npc.object.rotation.euler_angles().2.to_degrees();
        // TODO: Get index from LIST_EVENT for npc.quest_file_name
        Self {
            id: npc.object.object_id,
            direction: direction,
            quest_index: 0,
        }
    }
}
