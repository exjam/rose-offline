use crate::game::data::formats::STB;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::Deref;

#[derive(FromPrimitive)]
pub enum ItemClass {
    Ring = 171,
    Necklace = 172,
    Earring = 173,
    Arrow = 231,
    Arrow2 = 271,
    Bullet = 232,
    Throw = 233,
    Bullet2 = 253,
    NotUseBullet = 242,
    Shield = 261,
    SkillDoing = 313,
    SkillLearn = 314,
    RepairItem = 315,
    EventItem = 316,
    Fuel = 317,
    CartBody = 511,
    CastlerGearBody = 512,
    CartEngine = 521,
    CastleGearEngine = 522,
    CastleGearWeapon = 552,
}

#[derive(FromPrimitive)]
pub enum AbilityType {
    Gender = 2,
    Birthstone = 3,
    Class = 4,
    Union = 5,
    Rank = 6,
    Fame = 7,
    Face = 8,
    Hair = 9,

    Strength = 10,
    Dexterity = 11,
    Intelligence = 12,
    Concentration = 13,
    Charm = 14,
    Sense = 15,

    Health = 16,
    Mana = 17,
    Attack = 18,
    Defence = 19,
    Hit = 20,
    Resistence = 21,
    Avoid = 22,
    Speed = 23,
    AttackSpeed = 24,
    Weight = 25,
    Critical = 26,
    RecoverHealth = 27,
    RecoverMana = 28,

    SaveMana = 29,
    Experience = 30,
    Level = 31,
    BonusPoint = 32,
    PvpFlag = 33,
    TeamNumber = 34,
    HeadSize = 35,
    BodySize = 36,
    Skillpoint = 37,
    MaxHealth = 38,
    MaxMana = 39,
    Money = 40,

    PassiveAttackPowerUnarmed = 41,
    PassiveAttackPowerOneHanded = 42,
    PassiveAttackPowerTwoHanded = 43,
    PassiveAttackPowerBow = 44,
    PassiveAttackPowerGun = 45,
    PassiveAttackPowerStaffWand = 46,
    PassiveAttackPowerAutoBow = 47,
    PassiveAttackPowerKatarPair = 48,

    PassiveAttackSpeedBow = 49,
    PassiveAttackSpeedGun = 50,
    PassiveAttackSpeedPair = 51,

    PassiveMoveSpeed = 52,
    PassiveDefence = 53,
    PassiveMaxHealth = 54,
    PassiveMaxMana = 55,
    PassiveRecoverHealth = 56,
    PassiveRecoverMana = 57,
    PassiveWeight = 58,

    PassiveBuySkill = 59,
    PassiveSellSkill = 60,
    PassiveSaveMana = 61,
    PassiveMaxSummons = 62,
    PassiveDropRate = 63,

    Race = 71,
    DropRate = 72,
    FameG = 73,
    FameB = 74,
    CurrentPlanet = 75,
    Stamina = 76,
    Fuel = 77,
    Immunity = 78,

    UnionPoint1 = 81,
    UnionPoint2 = 82,
    UnionPoint3 = 83,
    UnionPoint4 = 84,
    UnionPoint5 = 85,
    UnionPoint6 = 86,
    UnionPoint7 = 87,
    UnionPoint8 = 88,
    UnionPoint9 = 89,
    UnionPoint10 = 90,

    GuildNumber = 91,
    GuildScore = 92,
    GuildPosition = 93,

    BankFree = 94,
    BankAddon = 95,
    StoreSkin = 96,
    VehicleHealth = 97,

    PassiveResistence = 98,
    PassiveHit = 99,
    PassiveCritical = 100,
    PassiveAvoid = 101,
    PassiveShield = 102,
    PassiveImmunity = 103,
}

pub struct StbItem(pub STB);

impl Deref for StbItem {
    type Target = STB;
    fn deref(&self) -> &STB {
        &self.0
    }
}

impl StbItem {
    pub fn get_item_class(&self, item_number: u16) -> Option<ItemClass> {
        self.0.try_get(item_number as usize, 4).and_then(|x| {
            x.parse::<u32>()
                .ok()
                .and_then(|x| FromPrimitive::from_u32(x))
        })
    }

    pub fn get_item_base_price(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 5)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_price_rate(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 6)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_weight(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 7)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_quality(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 8)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_icon_number(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 9)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_field_model(&self, item_number: u16) -> Option<&str> {
        self.0.try_get(item_number as usize, 10)
    }

    pub fn get_item_equip_sound(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 11)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_craft_skill_type(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 12)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_craft_skill_level(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 13)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_craft_material(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 14)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_craft_difficulty(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 15)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_equip_class_requirement(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 16)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_equip_union_requirement(&self, item_number: u16) -> Vec<u32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self
                .0
                .try_get(item_number as usize, 17 + i)
                .and_then(|x| x.parse::<u32>().ok())
            {
                if union != 0 {
                    requirements.push(union);
                }
            }
        }
        requirements
    }

    pub fn get_item_ability_requirement(&self, item_number: u16) -> Vec<(AbilityType, u32)> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get(item_number as usize, 19 + i * 2)
                .and_then(|x| {
                    x.parse::<u32>()
                        .ok()
                        .and_then(|x| FromPrimitive::from_u32(x))
                });
            let ability_value = self
                .0
                .try_get(item_number as usize, 20 + i * 2)
                .and_then(|x| x.parse::<u32>().ok());

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| requirements.push((ability_type, ability_value)))
            });
        }
        requirements
    }

    pub fn get_item_union_requirement(&self, item_number: u16) -> Vec<u32> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            if let Some(union) = self
                .0
                .try_get(item_number as usize, 23 + i * 3)
                .and_then(|x| x.parse::<u32>().ok())
            {
                if union != 0 {
                    requirements.push(union);
                }
            }
        }
        requirements
    }

    pub fn get_item_add_ability(&self, item_number: u16) -> Vec<(AbilityType, u32)> {
        let mut requirements = Vec::new();
        for i in 0..2 {
            let ability_type: Option<AbilityType> = self
                .0
                .try_get(item_number as usize, 24 + i * 3)
                .and_then(|x| {
                    x.parse::<u32>()
                        .ok()
                        .and_then(|x| FromPrimitive::from_u32(x))
                });
            let ability_value = self
                .0
                .try_get(item_number as usize, 25 + i * 3)
                .and_then(|x| x.parse::<u32>().ok());

            ability_type.map(|ability_type| {
                ability_value.map(|ability_value| requirements.push((ability_type, ability_value)))
            });
        }
        requirements
    }

    pub fn get_item_durability(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 29)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_rare_type(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 30)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_defence(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 31)
            .and_then(|x| x.parse::<u32>().ok())
    }

    pub fn get_item_resistence(&self, item_number: u16) -> Option<u32> {
        self.0
            .try_get(item_number as usize, 32)
            .and_then(|x| x.parse::<u32>().ok())
    }
}

pub struct StbItemBack(pub StbItem);

impl Deref for StbItemBack {
    type Target = StbItem;
    fn deref(&self) -> &StbItem {
        &self.0
    }
}

impl StbItemBack {
    pub fn get_back_move_speed(&self, item_number: u16) -> u32 {
        self.try_get(item_number as usize, 33)
            .and_then(|x| x.parse::<u32>().ok())
            .unwrap_or(0)
    }
}

pub struct StbItemFoot(pub StbItem);

impl Deref for StbItemFoot {
    type Target = StbItem;
    fn deref(&self) -> &StbItem {
        &self.0
    }
}

impl StbItemFoot {
    pub fn get_boots_move_speed(&self, item_number: u16) -> u32 {
        self.try_get(item_number as usize, 33)
            .and_then(|x| x.parse::<u32>().ok())
            .unwrap_or(0)
    }
}
