use crate::game::data::formats::ifo;
use crate::game::data::STB_EVENT;

#[derive(Clone)]
pub struct Npc {
    pub id: u32,
    pub quest_index: u16,
    pub direction: f32,
}

impl From<&ifo::Npc> for Npc {
    fn from(npc: &ifo::Npc) -> Self {
        let direction = npc.object.rotation.euler_angles().2.to_degrees();
        let quest_index = STB_EVENT.lookup_row_name(&npc.quest_file_name).unwrap_or(0) as u16;
        Self {
            id: npc.object.object_id,
            direction: direction,
            quest_index: quest_index,
        }
    }
}
