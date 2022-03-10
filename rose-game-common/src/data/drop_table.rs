use rose_data::{NpcId, ZoneId};

use crate::components::DroppedItem;

pub trait DropTable {
    #[allow(clippy::too_many_arguments)]
    fn get_drop(
        &self,
        world_drop_item_rate: i32,
        world_drop_money_rate: i32,
        npc_id: NpcId,
        zone_id: ZoneId,
        level_difference: i32,
        character_drop_rate: i32,
        character_charm: i32,
    ) -> Option<DroppedItem>;
}
