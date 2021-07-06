use std::sync::Arc;

use rand::Rng;

use crate::{
    data::{
        formats::{FileReader, StbFile, VfsIndex},
        item::{EquipmentItem, Item, ItemType},
        DropTable, ItemDatabase, ItemReference, NpcDatabase, NpcReference, ZoneReference,
    },
    game::components::DroppedItem,
};

pub struct DropTableData {
    item_database: Arc<ItemDatabase>,
    npc_database: Arc<NpcDatabase>,

    columns: usize,
    drop_table: Vec<i32>,
}

impl DropTableData {
    fn lookup_drop(&self, row: usize, column: usize) -> Option<i32> {
        self.drop_table.get(row * self.columns + column).cloned()
    }
}

impl DropTable for DropTableData {
    fn get_drop(
        &self,
        world_drop_item_rate: i32,
        world_drop_money_rate: i32,
        npc: NpcReference,
        zone: ZoneReference,
        level_difference: i32,
        character_drop_rate: i32,
        character_charm: i32,
    ) -> Option<DroppedItem> {
        let level_difference = level_difference.max(0);
        if level_difference > 10 {
            return None;
        }

        let npc_data = self.npc_database.get_npc(npc.0);
        let npc_drop_item_rate = npc_data.map_or(0, |n| n.drop_item_rate);
        let npc_drop_money_rate = npc_data.map_or(0, |n| n.drop_money_rate);
        let npc_level = npc_data.map_or(0, |n| n.level);

        let mut rng = rand::thread_rng();
        let drop_var = ((world_drop_item_rate as f32 + npc_drop_item_rate as f32
            - rng.gen_range(1..=100) as f32
            - (level_difference as f32 + 16.0) * 3.5
            - 10.0
            + character_drop_rate as f32)
            * 0.38) as i32;

        if drop_var <= 0 {
            return None;
        }

        if rng.gen_range(1..=100) <= npc_drop_money_rate {
            let amount =
                ((npc_level + 20) * (npc_level + drop_var + 40) * world_drop_money_rate) / 3200;
            if amount <= 0 {
                return None;
            }

            return Some(DroppedItem::Money(amount as usize));
        }

        let drop_table_row = if rng.gen_range(1..=100) <= npc_drop_item_rate {
            npc_data.map_or(zone.0, |n| n.drop_table_index as usize)
        } else {
            zone.0
        };
        let drop_table_column = rng.gen_range(0..drop_var.min(30)) as usize;
        let mut drop_value = self
            .lookup_drop(drop_table_row, drop_table_column)
            .unwrap_or(0);
        if (1..=4).contains(&drop_value) {
            let drop_table_column = (26 + drop_value * 5 + rng.gen_range(0..5)) as usize;
            drop_value = self
                .lookup_drop(drop_table_row, drop_table_column)
                .unwrap_or(0);
        }

        let item_reference = ItemReference::from_base1000(drop_value as u32).ok()?;
        match item_reference.item_type {
            ItemType::Face
            | ItemType::Head
            | ItemType::Body
            | ItemType::Hands
            | ItemType::Feet
            | ItemType::Back
            | ItemType::Jewellery
            | ItemType::Weapon
            | ItemType::SubWeapon => {
                let item_data = self.item_database.get_base_item(item_reference);
                let item_rare_type = item_data.map(|x| x.rare_type).unwrap_or(0);
                let mut item = EquipmentItem::new(&item_reference)?;

                match item_rare_type {
                    3 => {
                        item.gem = 100 + rng.gen_range(0..=40);
                    }
                    2 => {
                        item.has_socket = true;
                        item.is_appraised = true;
                    }
                    1 => {
                        let item_quality = item_data.map(|x| x.quality).unwrap_or(0);
                        if item_quality + 60 > rng.gen_range(0..400) {
                            item.has_socket = true;
                            item.is_appraised = true;
                        }
                    }
                    0 => {
                        if item.item.item_type != ItemType::Jewellery {
                            let item_op_rng = rng.gen_range(1..=100);
                            let item_op = (((npc_level as f32 * 0.4) as i32
                                + (npc_drop_item_rate - 35) * 4
                                + 80
                                - item_op_rng
                                + character_charm)
                                * 24
                                / (item_op_rng + 13))
                                - 100;
                            if item_op > 0 {
                                if npc_level < 230 {
                                    item.gem = (item_op % (npc_level + 70)) as u16;
                                } else {
                                    item.gem = (item_op % 301) as u16;
                                }
                                item.is_appraised = item.gem != 0;
                            }
                        }
                    }
                    _ => {}
                }

                let item_durability = item_data.map(|x| x.durability).unwrap_or(0);
                let durability = ((item_durability as f32
                    * (npc_level as f32 * 0.3 + npc_drop_item_rate as f32 * 2.0 + 320.0)
                    * 0.5)
                    / rng.gen_range(201..=300) as f32) as i32;
                item.durability = durability.min(100).max(0) as u8;

                let life = ((npc_drop_item_rate + 200) * 80) / rng.gen_range(31..=130);
                item.life = life.min(1000).max(0) as u16;

                if matches!(
                    item_reference.item_type,
                    ItemType::Weapon
                        | ItemType::SubWeapon
                        | ItemType::Head
                        | ItemType::Body
                        | ItemType::Hands
                        | ItemType::Feet
                ) {
                    let item_grade_rng = rng.gen_range(1..=100);
                    let item_grade = ((((npc_drop_item_rate - 5) * 3 + 150
                        - (npc_level as f32 * 1.5) as i32
                        - item_grade_rng
                        + character_charm) as f32
                        * 0.4)
                        / (item_grade_rng + 30) as f32) as i32
                        - 1;
                    item.grade = item_grade.min(3).max(0) as u8;
                }

                Some(DroppedItem::Item(Item::Equipment(item)))
            }
            ItemType::Consumable | ItemType::Vehicle => {
                Item::new(&item_reference, 1).map(DroppedItem::Item)
            }
            ItemType::Gem | ItemType::Material | ItemType::Quest => {
                let quantity = (1
                    + ((npc_level + 10) / 9 + rng.gen_range(1..=20) + character_drop_rate)
                        / (drop_var + 4)) as u32;
                Item::new(&item_reference, quantity.min(10)).map(DroppedItem::Item)
            }
            _ => None,
        }
    }
}

pub fn get_drop_table(
    vfs: &VfsIndex,
    item_database: Arc<ItemDatabase>,
    npc_database: Arc<NpcDatabase>,
) -> Option<Box<impl DropTable + Send + Sync>> {
    let file = vfs.open_file("3DDATA/STB/ITEM_DROP.STB")?;
    let stb = StbFile::read(FileReader::from(&file)).ok()?;
    let rows = stb.rows();
    let columns = stb.columns();
    let mut drop_table = Vec::with_capacity(rows * columns);

    for y in 0..rows {
        for x in 0..columns {
            drop_table.push(stb.get_int(y, x));
        }
    }

    Some(Box::new(DropTableData {
        item_database,
        npc_database,
        columns,
        drop_table,
    }))
}
