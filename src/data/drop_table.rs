use crate::{
    data::{NpcId, ZoneId},
    game::components::DroppedItem,
};

pub trait DropTable {
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
