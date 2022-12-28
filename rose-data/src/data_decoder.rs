use crate::{
    AbilityType, AmmoIndex, EquipmentIndex, ItemClass, ItemReference, ItemType, VehiclePartIndex, ClanMemberPosition
};

pub trait DataDecoder {
    fn decode_item_base1000(&self, id: usize) -> Option<ItemReference>;
    fn decode_ability_type(&self, id: usize) -> Option<AbilityType>;
    fn decode_item_type(&self, id: usize) -> Option<ItemType>;
    fn decode_item_class(&self, id: usize) -> Option<ItemClass>;

    fn decode_equipment_index(&self, id: usize) -> Option<EquipmentIndex>;
    fn decode_vehicle_part_index(&self, id: usize) -> Option<VehiclePartIndex>;
    fn decode_ammo_index(&self, id: usize) -> Option<AmmoIndex>;

    fn encode_clan_member_position(&self, position: ClanMemberPosition) -> Option<usize>;
}
