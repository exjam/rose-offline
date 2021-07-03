#[derive(Debug, PartialEq)]
pub enum DamageCategory {
    Character,
    Npc,
}

#[derive(Debug, PartialEq)]
pub enum DamageType {
    Physical,
    Magic,
}

#[derive(Debug)]
pub struct AbilityValues {
    pub damage_category: DamageCategory,
    pub level: i32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub strength: u16,
    pub dexterity: u16,
    pub intelligence: u16,
    pub concentration: u16,
    pub charm: u16,
    pub sense: u16,
    pub max_health: i32,
    pub max_mana: i32,
    pub additional_health_recovery: i32,
    pub additional_mana_recovery: i32,
    pub attack_damage_type: DamageType,
    pub attack_power: i32,
    pub attack_speed: i32,
    pub passive_attack_speed: i32,
    pub attack_range: i32,
    pub hit: i32,
    pub defence: i32,
    pub resistance: i32,
    pub critical: i32,
    pub avoid: i32,
    pub max_damage_sources: usize,
}
