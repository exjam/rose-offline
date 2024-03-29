use std::str::FromStr;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use rose_data::{
    AbilityType, AmmoIndex, ClanMemberPosition, DataDecoder, EffectBulletMoveType, EquipmentIndex,
    ItemClass, ItemReference, ItemType, SkillActionMode, SkillBasicCommand, SkillTargetFilter,
    SkillType, StatusEffectClearedByType, StatusEffectType, VehiclePartIndex, VehicleType,
};

macro_rules! impl_conversions {
    (
        $irose_class:ident, $common_class:ident, $decode_fn:ident
    ) => {
        impl FromStr for $irose_class {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<usize>().map_err(|_| ())?;
                FromPrimitive::from_usize(value).ok_or(())
            }
        }

        impl TryFrom<$irose_class> for $common_class {
            type Error = ();

            fn try_from(value: $irose_class) -> Result<Self, Self::Error> {
                $decode_fn(value as usize).ok_or(())
            }
        }
    };
}

#[derive(Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum IroseAbilityType {
    Gender = 2,
    Birthstone = 3,
    Job = 4,
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
    Resistance = 21,
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

    PassiveResistance = 98,
    PassiveHit = 99,
    PassiveCritical = 100,
    PassiveAvoid = 101,
    PassiveShieldDefence = 102,
    PassiveImmunity = 103,
}
impl_conversions!(IroseAbilityType, AbilityType, decode_ability_type);

#[derive(FromPrimitive)]
pub enum IroseEffectBulletMoveType {
    Linear = 1,
    Parabola = 2,
    Immediate = 3,
}
impl_conversions!(
    IroseEffectBulletMoveType,
    EffectBulletMoveType,
    decode_effect_bullet_move_type
);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseItemClass {
    Unknown = 0,

    FaceMask = 111,
    FaceGlasses = 112,
    FaceEtc = 113,

    Helmet = 121,
    MagicHat = 122,
    Hat = 123,
    HairAccessory = 124,

    CombatUniform = 131,
    MagicClothes = 132,
    CasualClothes = 133,

    Gauntlet = 141,
    MagicGlove = 142,
    Glove = 143,

    Boots = 151,
    MagicBoots = 152,
    Shoes = 153,

    BackArmor = 161,
    Bag = 162,
    Wings = 163,
    ArrowBox = 164,
    BulletBox = 165,
    ShellBox = 166,

    Ring = 171,
    Necklace = 172,
    Earring = 173,

    OneHandedSword = 211,
    OneHandedBlunt = 212,

    TwoHandedSword = 221,
    Spear = 222,
    TwoHandedAxe = 223,

    Bow = 231,
    Gun = 232,
    Launcher = 233,

    MagicStaff = 241,
    MagicWand = 242,

    Katar = 251,
    DualSwords = 252,
    DualGuns = 253,

    Shield = 261,
    SupportTool = 262,

    Crossbow = 271,

    Medicine = 311,
    Food = 312,
    MagicItem = 313,
    SkillBook = 314,
    RepairTool = 315,
    QuestScroll = 316,
    EngineFuel = 317,
    AutomaticConsumption = 320,
    TimeCoupon = 321,

    Jewel = 411,
    WorkOfArt = 412,

    Metal = 421,
    OtherworldlyMetal = 422,
    StoneMaterial = 423,
    WoodenMaterial = 424,
    Leather = 425,
    Cloth = 426,
    RefiningMaterial = 427,
    Chemicals = 428,
    Material = 429,
    GatheredGoods = 430,

    Arrow = 431,
    Bullet = 432,
    Shell = 433,

    QuestItems = 441,
    Certification = 442,

    CartBody = 511,
    CastleGearBody = 512,

    CartEngine = 521,
    CastleGearEngine = 522,

    CartWheels = 531,
    CastleGearLeg = 532,

    CartAccessory = 551,
    CastleGearWeapon = 552,
}
impl_conversions!(IroseItemClass, ItemClass, decode_item_class);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseItemType {
    Face = 1,
    Head = 2,
    Body = 3,
    Hands = 4,
    Feet = 5,
    Back = 6,
    Jewellery = 7,
    Weapon = 8,
    SubWeapon = 9,
    Consumable = 10,
    Gem = 11,
    Material = 12,
    Quest = 13,
    Vehicle = 14,
}
impl_conversions!(IroseItemType, ItemType, decode_item_type);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseEquipmentIndex {
    Face = 1,
    Head = 2,
    Body = 3,
    Back = 4,
    Hands = 5,
    Feet = 6,
    Weapon = 7,
    SubWeapon = 8,
    Necklace = 9,
    Ring = 10,
    Earring = 11,
}
impl_conversions!(IroseEquipmentIndex, EquipmentIndex, decode_equipment_index);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseVehiclePartIndex {
    Body = 0,
    Engine = 1,
    Leg = 2,
    Arms = 3,
}
impl_conversions!(
    IroseVehiclePartIndex,
    VehiclePartIndex,
    decode_vehicle_part_index
);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseVehicleType {
    Cart = 21,
    CastleGear = 31,
}
impl_conversions!(IroseVehicleType, VehicleType, decode_vehicle_type);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseAmmoIndex {
    Arrow = 0,
    Bullet = 1,
    Throw = 2,
}
impl_conversions!(IroseAmmoIndex, AmmoIndex, decode_ammo_index);

#[derive(FromPrimitive)]
pub enum IroseStatusEffectType {
    IncreaseHp = 1,
    IncreaseMp = 2,
    Poisoned = 3,
    IncreaseMaxHp = 4,
    IncreaseMaxMp = 5,
    IncreaseMoveSpeed = 6,
    DecreaseMoveSpeed = 7,
    IncreaseAttackSpeed = 8,
    DecreaseAttackSpeed = 9,
    IncreaseAttackPower = 10,
    DecreaseAttackPower = 11,
    IncreaseDefence = 12,
    DecreaseDefence = 13,
    IncreaseResistance = 14,
    DecreaseResistance = 15,
    IncreaseHit = 16,
    DecreaseHit = 17,
    IncreaseCritical = 18,
    DecreaseCritical = 19,
    IncreaseAvoid = 20,
    DecreaseAvoid = 21,
    Dumb = 22,
    Sleep = 23,
    Fainting = 24,
    Disguise = 25,
    Transparent = 26,
    ShieldDamage = 27,
    AdditionalDamageRate = 28,
    DecreaseLifeTime = 29,
    ClearGood = 30,
    ClearBad = 31,
    ClearAll = 32,
    ClearInvisible = 33,
    Taunt = 34,
    Revive = 35,
}
impl_conversions!(
    IroseStatusEffectType,
    StatusEffectType,
    decode_status_effect_type
);

#[derive(FromPrimitive)]
pub enum IroseStatusEffectClearedByType {
    Good = 0,
    Bad = 1,
    None = 2,
}
impl_conversions!(
    IroseStatusEffectClearedByType,
    StatusEffectClearedByType,
    decode_status_effect_cleared_by_type
);

#[derive(FromPrimitive)]
pub enum IroseSkillActionMode {
    Stop = 0,
    Attack = 1,
    Restore = 2,
}
impl_conversions!(
    IroseSkillActionMode,
    SkillActionMode,
    decode_skill_action_mode
);

#[derive(FromPrimitive)]
pub enum IroseSkillBasicCommand {
    Sit = 1,
    PickupItem = 2,
    Jump = 3,
    AirJump = 4,
    AutoTarget = 5,
    Attack = 6,
    DriveVehicle = 7,
    AddFriend = 8,
    PartyInvite = 9,
    Trade = 10,
    PrivateStore = 11,
    SelfTarget = 12,
    VehiclePassengerInvite = 13,
}
impl_conversions!(
    IroseSkillBasicCommand,
    SkillBasicCommand,
    decode_skill_basic_command
);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseSkillTargetFilter {
    OnlySelf = 0,
    Group = 1,
    Guild = 2,
    Allied = 3,
    Monster = 4,
    Enemy = 5,
    EnemyCharacter = 6,
    Character = 7,
    CharacterOrMonster = 8,
    DeadAlliedCharacter = 9,
    EnemyMonster = 10,
}
impl_conversions!(
    IroseSkillTargetFilter,
    SkillTargetFilter,
    decode_skill_target_filter
);

#[derive(Copy, Clone, Debug, FromPrimitive)]
pub enum IroseSkillPageType {
    Basic = 0,
    Active = 1,
    Passive = 2,
    Clan = 3,
}

impl FromStr for IroseSkillPageType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<usize>().map_err(|_| ())?;
        FromPrimitive::from_usize(value).ok_or(())
    }
}

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseSkillType {
    BasicAction = 1,
    CreateWindow = 2,
    Immediate = 3,
    EnforceWeapon = 4,
    EnforceBullet = 5,
    FireBullet = 6,
    AreaTarget = 7,
    SelfBoundDuration = 8,
    TargetBoundDuration = 9,
    SelfBound = 10,
    TargetBound = 11,
    SelfStateDuration = 12,
    TargetStateDuration = 13,
    SummonPet = 14,
    Passive = 15,
    Emote = 16,
    SelfDamage = 17,
    Warp = 18,
    SelfAndTarget = 19,
    Resurrection = 20,
}
impl_conversions!(IroseSkillType, SkillType, decode_skill_type);

#[derive(FromPrimitive, ToPrimitive)]
pub enum IroseClanMemberPosition {
    Penalty = 0,
    Junior = 1,
    Senior = 2,
    Veteran = 3,
    Commander = 4,
    DeputyMaster = 5,
    Master = 6,
}
impl_conversions!(
    IroseClanMemberPosition,
    ClanMemberPosition,
    decode_clan_member_position
);

struct IroseDataDecoder {}

impl DataDecoder for IroseDataDecoder {
    fn decode_ability_type(&self, id: usize) -> Option<AbilityType> {
        decode_ability_type(id)
    }

    fn decode_item_type(&self, id: usize) -> Option<ItemType> {
        decode_item_type(id)
    }

    fn decode_item_class(&self, id: usize) -> Option<ItemClass> {
        decode_item_class(id)
    }

    fn decode_item_base1000(&self, id: usize) -> Option<ItemReference> {
        decode_item_base1000(id)
    }

    fn decode_item_reference(&self, item_number: usize, item_type: usize) -> Option<ItemReference> {
        decode_item_reference(item_number, item_type)
    }

    fn decode_equipment_index(&self, id: usize) -> Option<EquipmentIndex> {
        decode_equipment_index(id)
    }

    fn decode_vehicle_part_index(&self, id: usize) -> Option<VehiclePartIndex> {
        decode_vehicle_part_index(id)
    }

    fn decode_ammo_index(&self, id: usize) -> Option<AmmoIndex> {
        decode_ammo_index(id)
    }

    fn encode_clan_member_position(&self, position: ClanMemberPosition) -> Option<usize> {
        encode_clan_member_position(position)
    }
}

pub fn get_data_decoder() -> Box<impl DataDecoder + Send + Sync> {
    Box::new(IroseDataDecoder {})
}

pub fn decode_item_base1000(id: usize) -> Option<ItemReference> {
    if id == 0 {
        None
    } else {
        let item_type = decode_item_type(id / 1000)?;
        let item_number = id % 1000;
        if item_number == 0 {
            None
        } else {
            Some(ItemReference::new(item_type, item_number))
        }
    }
}

pub fn decode_item_reference(item_number: usize, item_type: usize) -> Option<ItemReference> {
    let item_type = decode_item_type(item_type)?;
    if item_number == 0 {
        None
    } else {
        Some(ItemReference::new(item_type, item_number))
    }
}

pub fn decode_ability_type(id: usize) -> Option<AbilityType> {
    match FromPrimitive::from_usize(id)? {
        IroseAbilityType::Gender => Some(AbilityType::Gender),
        IroseAbilityType::Birthstone => Some(AbilityType::Birthstone),
        IroseAbilityType::Job => Some(AbilityType::Job),
        IroseAbilityType::Union => Some(AbilityType::Union),
        IroseAbilityType::Rank => Some(AbilityType::Rank),
        IroseAbilityType::Fame => Some(AbilityType::Fame),
        IroseAbilityType::Face => Some(AbilityType::Face),
        IroseAbilityType::Hair => Some(AbilityType::Hair),
        IroseAbilityType::Strength => Some(AbilityType::Strength),
        IroseAbilityType::Dexterity => Some(AbilityType::Dexterity),
        IroseAbilityType::Intelligence => Some(AbilityType::Intelligence),
        IroseAbilityType::Concentration => Some(AbilityType::Concentration),
        IroseAbilityType::Charm => Some(AbilityType::Charm),
        IroseAbilityType::Sense => Some(AbilityType::Sense),
        IroseAbilityType::Health => Some(AbilityType::Health),
        IroseAbilityType::Mana => Some(AbilityType::Mana),
        IroseAbilityType::Attack => Some(AbilityType::Attack),
        IroseAbilityType::Defence => Some(AbilityType::Defence),
        IroseAbilityType::Hit => Some(AbilityType::Hit),
        IroseAbilityType::Resistance => Some(AbilityType::Resistance),
        IroseAbilityType::Avoid => Some(AbilityType::Avoid),
        IroseAbilityType::Speed => Some(AbilityType::Speed),
        IroseAbilityType::AttackSpeed => Some(AbilityType::AttackSpeed),
        IroseAbilityType::Weight => Some(AbilityType::Weight),
        IroseAbilityType::Critical => Some(AbilityType::Critical),
        IroseAbilityType::RecoverHealth => Some(AbilityType::RecoverHealth),
        IroseAbilityType::RecoverMana => Some(AbilityType::RecoverMana),
        IroseAbilityType::SaveMana => Some(AbilityType::SaveMana),
        IroseAbilityType::Experience => Some(AbilityType::Experience),
        IroseAbilityType::Level => Some(AbilityType::Level),
        IroseAbilityType::BonusPoint => Some(AbilityType::BonusPoint),
        IroseAbilityType::PvpFlag => Some(AbilityType::PvpFlag),
        IroseAbilityType::TeamNumber => Some(AbilityType::TeamNumber),
        IroseAbilityType::HeadSize => Some(AbilityType::HeadSize),
        IroseAbilityType::BodySize => Some(AbilityType::BodySize),
        IroseAbilityType::Skillpoint => Some(AbilityType::Skillpoint),
        IroseAbilityType::MaxHealth => Some(AbilityType::MaxHealth),
        IroseAbilityType::MaxMana => Some(AbilityType::MaxMana),
        IroseAbilityType::Money => Some(AbilityType::Money),
        IroseAbilityType::PassiveAttackPowerUnarmed => Some(AbilityType::PassiveAttackPowerUnarmed),
        IroseAbilityType::PassiveAttackPowerOneHanded => {
            Some(AbilityType::PassiveAttackPowerOneHanded)
        }
        IroseAbilityType::PassiveAttackPowerTwoHanded => {
            Some(AbilityType::PassiveAttackPowerTwoHanded)
        }
        IroseAbilityType::PassiveAttackPowerBow => Some(AbilityType::PassiveAttackPowerBow),
        IroseAbilityType::PassiveAttackPowerGun => Some(AbilityType::PassiveAttackPowerGun),
        IroseAbilityType::PassiveAttackPowerStaffWand => {
            Some(AbilityType::PassiveAttackPowerStaffWand)
        }
        IroseAbilityType::PassiveAttackPowerAutoBow => Some(AbilityType::PassiveAttackPowerAutoBow),
        IroseAbilityType::PassiveAttackPowerKatarPair => {
            Some(AbilityType::PassiveAttackPowerKatarPair)
        }
        IroseAbilityType::PassiveAttackSpeedBow => Some(AbilityType::PassiveAttackSpeedBow),
        IroseAbilityType::PassiveAttackSpeedGun => Some(AbilityType::PassiveAttackSpeedGun),
        IroseAbilityType::PassiveAttackSpeedPair => Some(AbilityType::PassiveAttackSpeedPair),
        IroseAbilityType::PassiveMoveSpeed => Some(AbilityType::PassiveMoveSpeed),
        IroseAbilityType::PassiveDefence => Some(AbilityType::PassiveDefence),
        IroseAbilityType::PassiveMaxHealth => Some(AbilityType::PassiveMaxHealth),
        IroseAbilityType::PassiveMaxMana => Some(AbilityType::PassiveMaxMana),
        IroseAbilityType::PassiveRecoverHealth => Some(AbilityType::PassiveRecoverHealth),
        IroseAbilityType::PassiveRecoverMana => Some(AbilityType::PassiveRecoverMana),
        IroseAbilityType::PassiveWeight => Some(AbilityType::PassiveWeight),
        IroseAbilityType::PassiveBuySkill => Some(AbilityType::PassiveBuySkill),
        IroseAbilityType::PassiveSellSkill => Some(AbilityType::PassiveSellSkill),
        IroseAbilityType::PassiveSaveMana => Some(AbilityType::PassiveSaveMana),
        IroseAbilityType::PassiveMaxSummons => Some(AbilityType::PassiveMaxSummons),
        IroseAbilityType::PassiveDropRate => Some(AbilityType::PassiveDropRate),
        IroseAbilityType::Race => Some(AbilityType::Race),
        IroseAbilityType::DropRate => Some(AbilityType::DropRate),
        IroseAbilityType::FameG => Some(AbilityType::FameG),
        IroseAbilityType::FameB => Some(AbilityType::FameB),
        IroseAbilityType::CurrentPlanet => Some(AbilityType::CurrentPlanet),
        IroseAbilityType::Stamina => Some(AbilityType::Stamina),
        IroseAbilityType::Fuel => Some(AbilityType::Fuel),
        IroseAbilityType::Immunity => Some(AbilityType::Immunity),
        IroseAbilityType::UnionPoint1 => Some(AbilityType::UnionPoint1),
        IroseAbilityType::UnionPoint2 => Some(AbilityType::UnionPoint2),
        IroseAbilityType::UnionPoint3 => Some(AbilityType::UnionPoint3),
        IroseAbilityType::UnionPoint4 => Some(AbilityType::UnionPoint4),
        IroseAbilityType::UnionPoint5 => Some(AbilityType::UnionPoint5),
        IroseAbilityType::UnionPoint6 => Some(AbilityType::UnionPoint6),
        IroseAbilityType::UnionPoint7 => Some(AbilityType::UnionPoint7),
        IroseAbilityType::UnionPoint8 => Some(AbilityType::UnionPoint8),
        IroseAbilityType::UnionPoint9 => Some(AbilityType::UnionPoint9),
        IroseAbilityType::UnionPoint10 => Some(AbilityType::UnionPoint10),
        IroseAbilityType::GuildNumber => Some(AbilityType::GuildNumber),
        IroseAbilityType::GuildScore => Some(AbilityType::GuildScore),
        IroseAbilityType::GuildPosition => Some(AbilityType::GuildPosition),
        IroseAbilityType::BankFree => Some(AbilityType::BankFree),
        IroseAbilityType::BankAddon => Some(AbilityType::BankAddon),
        IroseAbilityType::StoreSkin => Some(AbilityType::StoreSkin),
        IroseAbilityType::VehicleHealth => Some(AbilityType::VehicleHealth),
        IroseAbilityType::PassiveResistance => Some(AbilityType::PassiveResistance),
        IroseAbilityType::PassiveHit => Some(AbilityType::PassiveHit),
        IroseAbilityType::PassiveCritical => Some(AbilityType::PassiveCritical),
        IroseAbilityType::PassiveAvoid => Some(AbilityType::PassiveAvoid),
        IroseAbilityType::PassiveShieldDefence => Some(AbilityType::PassiveShieldDefence),
        IroseAbilityType::PassiveImmunity => Some(AbilityType::PassiveImmunity),
    }
}

pub fn encode_ability_type(from: AbilityType) -> Option<usize> {
    match from {
        AbilityType::Gender => IroseAbilityType::Gender.to_usize(),
        AbilityType::Birthstone => IroseAbilityType::Birthstone.to_usize(),
        AbilityType::Job => IroseAbilityType::Job.to_usize(),
        AbilityType::Union => IroseAbilityType::Union.to_usize(),
        AbilityType::Rank => IroseAbilityType::Rank.to_usize(),
        AbilityType::Fame => IroseAbilityType::Fame.to_usize(),
        AbilityType::Face => IroseAbilityType::Face.to_usize(),
        AbilityType::Hair => IroseAbilityType::Hair.to_usize(),
        AbilityType::Strength => IroseAbilityType::Strength.to_usize(),
        AbilityType::Dexterity => IroseAbilityType::Dexterity.to_usize(),
        AbilityType::Intelligence => IroseAbilityType::Intelligence.to_usize(),
        AbilityType::Concentration => IroseAbilityType::Concentration.to_usize(),
        AbilityType::Charm => IroseAbilityType::Charm.to_usize(),
        AbilityType::Sense => IroseAbilityType::Sense.to_usize(),
        AbilityType::Health => IroseAbilityType::Health.to_usize(),
        AbilityType::Mana => IroseAbilityType::Mana.to_usize(),
        AbilityType::Attack => IroseAbilityType::Attack.to_usize(),
        AbilityType::Defence => IroseAbilityType::Defence.to_usize(),
        AbilityType::Hit => IroseAbilityType::Hit.to_usize(),
        AbilityType::Resistance => IroseAbilityType::Resistance.to_usize(),
        AbilityType::Avoid => IroseAbilityType::Avoid.to_usize(),
        AbilityType::Speed => IroseAbilityType::Speed.to_usize(),
        AbilityType::AttackSpeed => IroseAbilityType::AttackSpeed.to_usize(),
        AbilityType::Weight => IroseAbilityType::Weight.to_usize(),
        AbilityType::Critical => IroseAbilityType::Critical.to_usize(),
        AbilityType::RecoverHealth => IroseAbilityType::RecoverHealth.to_usize(),
        AbilityType::RecoverMana => IroseAbilityType::RecoverMana.to_usize(),
        AbilityType::SaveMana => IroseAbilityType::SaveMana.to_usize(),
        AbilityType::Experience => IroseAbilityType::Experience.to_usize(),
        AbilityType::Level => IroseAbilityType::Level.to_usize(),
        AbilityType::BonusPoint => IroseAbilityType::BonusPoint.to_usize(),
        AbilityType::PvpFlag => IroseAbilityType::PvpFlag.to_usize(),
        AbilityType::TeamNumber => IroseAbilityType::TeamNumber.to_usize(),
        AbilityType::HeadSize => IroseAbilityType::HeadSize.to_usize(),
        AbilityType::BodySize => IroseAbilityType::BodySize.to_usize(),
        AbilityType::Skillpoint => IroseAbilityType::Skillpoint.to_usize(),
        AbilityType::MaxHealth => IroseAbilityType::MaxHealth.to_usize(),
        AbilityType::MaxMana => IroseAbilityType::MaxMana.to_usize(),
        AbilityType::Money => IroseAbilityType::Money.to_usize(),
        AbilityType::PassiveAttackPowerUnarmed => {
            IroseAbilityType::PassiveAttackPowerUnarmed.to_usize()
        }
        AbilityType::PassiveAttackPowerOneHanded => {
            IroseAbilityType::PassiveAttackPowerOneHanded.to_usize()
        }
        AbilityType::PassiveAttackPowerTwoHanded => {
            IroseAbilityType::PassiveAttackPowerTwoHanded.to_usize()
        }
        AbilityType::PassiveAttackPowerBow => IroseAbilityType::PassiveAttackPowerBow.to_usize(),
        AbilityType::PassiveAttackPowerGun => IroseAbilityType::PassiveAttackPowerGun.to_usize(),
        AbilityType::PassiveAttackPowerStaffWand => {
            IroseAbilityType::PassiveAttackPowerStaffWand.to_usize()
        }
        AbilityType::PassiveAttackPowerAutoBow => {
            IroseAbilityType::PassiveAttackPowerAutoBow.to_usize()
        }
        AbilityType::PassiveAttackPowerKatarPair => {
            IroseAbilityType::PassiveAttackPowerKatarPair.to_usize()
        }
        AbilityType::PassiveAttackSpeedBow => IroseAbilityType::PassiveAttackSpeedBow.to_usize(),
        AbilityType::PassiveAttackSpeedGun => IroseAbilityType::PassiveAttackSpeedGun.to_usize(),
        AbilityType::PassiveAttackSpeedPair => IroseAbilityType::PassiveAttackSpeedPair.to_usize(),
        AbilityType::PassiveMoveSpeed => IroseAbilityType::PassiveMoveSpeed.to_usize(),
        AbilityType::PassiveDefence => IroseAbilityType::PassiveDefence.to_usize(),
        AbilityType::PassiveMaxHealth => IroseAbilityType::PassiveMaxHealth.to_usize(),
        AbilityType::PassiveMaxMana => IroseAbilityType::PassiveMaxMana.to_usize(),
        AbilityType::PassiveRecoverHealth => IroseAbilityType::PassiveRecoverHealth.to_usize(),
        AbilityType::PassiveRecoverMana => IroseAbilityType::PassiveRecoverMana.to_usize(),
        AbilityType::PassiveWeight => IroseAbilityType::PassiveWeight.to_usize(),
        AbilityType::PassiveBuySkill => IroseAbilityType::PassiveBuySkill.to_usize(),
        AbilityType::PassiveSellSkill => IroseAbilityType::PassiveSellSkill.to_usize(),
        AbilityType::PassiveSaveMana => IroseAbilityType::PassiveSaveMana.to_usize(),
        AbilityType::PassiveMaxSummons => IroseAbilityType::PassiveMaxSummons.to_usize(),
        AbilityType::PassiveDropRate => IroseAbilityType::PassiveDropRate.to_usize(),
        AbilityType::Race => IroseAbilityType::Race.to_usize(),
        AbilityType::DropRate => IroseAbilityType::DropRate.to_usize(),
        AbilityType::FameG => IroseAbilityType::FameG.to_usize(),
        AbilityType::FameB => IroseAbilityType::FameB.to_usize(),
        AbilityType::CurrentPlanet => IroseAbilityType::CurrentPlanet.to_usize(),
        AbilityType::Stamina => IroseAbilityType::Stamina.to_usize(),
        AbilityType::Fuel => IroseAbilityType::Fuel.to_usize(),
        AbilityType::Immunity => IroseAbilityType::Immunity.to_usize(),
        AbilityType::UnionPoint1 => IroseAbilityType::UnionPoint1.to_usize(),
        AbilityType::UnionPoint2 => IroseAbilityType::UnionPoint2.to_usize(),
        AbilityType::UnionPoint3 => IroseAbilityType::UnionPoint3.to_usize(),
        AbilityType::UnionPoint4 => IroseAbilityType::UnionPoint4.to_usize(),
        AbilityType::UnionPoint5 => IroseAbilityType::UnionPoint5.to_usize(),
        AbilityType::UnionPoint6 => IroseAbilityType::UnionPoint6.to_usize(),
        AbilityType::UnionPoint7 => IroseAbilityType::UnionPoint7.to_usize(),
        AbilityType::UnionPoint8 => IroseAbilityType::UnionPoint8.to_usize(),
        AbilityType::UnionPoint9 => IroseAbilityType::UnionPoint9.to_usize(),
        AbilityType::UnionPoint10 => IroseAbilityType::UnionPoint10.to_usize(),
        AbilityType::GuildNumber => IroseAbilityType::GuildNumber.to_usize(),
        AbilityType::GuildScore => IroseAbilityType::GuildScore.to_usize(),
        AbilityType::GuildPosition => IroseAbilityType::GuildPosition.to_usize(),
        AbilityType::BankFree => IroseAbilityType::BankFree.to_usize(),
        AbilityType::BankAddon => IroseAbilityType::BankAddon.to_usize(),
        AbilityType::StoreSkin => IroseAbilityType::StoreSkin.to_usize(),
        AbilityType::VehicleHealth => IroseAbilityType::VehicleHealth.to_usize(),
        AbilityType::PassiveResistance => IroseAbilityType::PassiveResistance.to_usize(),
        AbilityType::PassiveHit => IroseAbilityType::PassiveHit.to_usize(),
        AbilityType::PassiveCritical => IroseAbilityType::PassiveCritical.to_usize(),
        AbilityType::PassiveAvoid => IroseAbilityType::PassiveAvoid.to_usize(),
        AbilityType::PassiveShieldDefence => IroseAbilityType::PassiveShieldDefence.to_usize(),
        AbilityType::PassiveImmunity => IroseAbilityType::PassiveImmunity.to_usize(),
    }
}

pub fn decode_effect_bullet_move_type(id: usize) -> Option<EffectBulletMoveType> {
    match FromPrimitive::from_usize(id)? {
        IroseEffectBulletMoveType::Linear => Some(EffectBulletMoveType::Linear),
        IroseEffectBulletMoveType::Parabola => Some(EffectBulletMoveType::Parabola),
        IroseEffectBulletMoveType::Immediate => Some(EffectBulletMoveType::Immediate),
    }
}

pub fn decode_item_class(id: usize) -> Option<ItemClass> {
    match FromPrimitive::from_usize(id)? {
        IroseItemClass::Unknown => Some(ItemClass::Unknown),
        IroseItemClass::FaceMask => Some(ItemClass::FaceMask),
        IroseItemClass::FaceGlasses => Some(ItemClass::FaceGlasses),
        IroseItemClass::FaceEtc => Some(ItemClass::FaceEtc),
        IroseItemClass::Helmet => Some(ItemClass::Helmet),
        IroseItemClass::MagicHat => Some(ItemClass::MagicHat),
        IroseItemClass::Hat => Some(ItemClass::Hat),
        IroseItemClass::HairAccessory => Some(ItemClass::HairAccessory),
        IroseItemClass::CombatUniform => Some(ItemClass::CombatUniform),
        IroseItemClass::MagicClothes => Some(ItemClass::MagicClothes),
        IroseItemClass::CasualClothes => Some(ItemClass::CasualClothes),
        IroseItemClass::Gauntlet => Some(ItemClass::Gauntlet),
        IroseItemClass::MagicGlove => Some(ItemClass::MagicGlove),
        IroseItemClass::Glove => Some(ItemClass::Glove),
        IroseItemClass::Boots => Some(ItemClass::Boots),
        IroseItemClass::MagicBoots => Some(ItemClass::MagicBoots),
        IroseItemClass::Shoes => Some(ItemClass::Shoes),
        IroseItemClass::BackArmor => Some(ItemClass::BackArmor),
        IroseItemClass::Bag => Some(ItemClass::Bag),
        IroseItemClass::Wings => Some(ItemClass::Wings),
        IroseItemClass::ArrowBox => Some(ItemClass::ArrowBox),
        IroseItemClass::BulletBox => Some(ItemClass::BulletBox),
        IroseItemClass::ShellBox => Some(ItemClass::ShellBox),
        IroseItemClass::Ring => Some(ItemClass::Ring),
        IroseItemClass::Necklace => Some(ItemClass::Necklace),
        IroseItemClass::Earring => Some(ItemClass::Earring),
        IroseItemClass::OneHandedSword => Some(ItemClass::OneHandedSword),
        IroseItemClass::OneHandedBlunt => Some(ItemClass::OneHandedBlunt),
        IroseItemClass::TwoHandedSword => Some(ItemClass::TwoHandedSword),
        IroseItemClass::Spear => Some(ItemClass::Spear),
        IroseItemClass::TwoHandedAxe => Some(ItemClass::TwoHandedAxe),
        IroseItemClass::Bow => Some(ItemClass::Bow),
        IroseItemClass::Gun => Some(ItemClass::Gun),
        IroseItemClass::Launcher => Some(ItemClass::Launcher),
        IroseItemClass::MagicStaff => Some(ItemClass::MagicStaff),
        IroseItemClass::MagicWand => Some(ItemClass::MagicWand),
        IroseItemClass::Katar => Some(ItemClass::Katar),
        IroseItemClass::DualSwords => Some(ItemClass::DualSwords),
        IroseItemClass::DualGuns => Some(ItemClass::DualGuns),
        IroseItemClass::Shield => Some(ItemClass::Shield),
        IroseItemClass::SupportTool => Some(ItemClass::SupportTool),
        IroseItemClass::Crossbow => Some(ItemClass::Crossbow),
        IroseItemClass::Medicine => Some(ItemClass::Medicine),
        IroseItemClass::Food => Some(ItemClass::Food),
        IroseItemClass::MagicItem => Some(ItemClass::MagicItem),
        IroseItemClass::SkillBook => Some(ItemClass::SkillBook),
        IroseItemClass::RepairTool => Some(ItemClass::RepairTool),
        IroseItemClass::QuestScroll => Some(ItemClass::QuestScroll),
        IroseItemClass::EngineFuel => Some(ItemClass::EngineFuel),
        IroseItemClass::AutomaticConsumption => Some(ItemClass::AutomaticConsumption),
        IroseItemClass::TimeCoupon => Some(ItemClass::TimeCoupon),
        IroseItemClass::Jewel => Some(ItemClass::Jewel),
        IroseItemClass::WorkOfArt => Some(ItemClass::WorkOfArt),
        IroseItemClass::Metal => Some(ItemClass::Metal),
        IroseItemClass::OtherworldlyMetal => Some(ItemClass::OtherworldlyMetal),
        IroseItemClass::StoneMaterial => Some(ItemClass::StoneMaterial),
        IroseItemClass::WoodenMaterial => Some(ItemClass::WoodenMaterial),
        IroseItemClass::Leather => Some(ItemClass::Leather),
        IroseItemClass::Cloth => Some(ItemClass::Cloth),
        IroseItemClass::RefiningMaterial => Some(ItemClass::RefiningMaterial),
        IroseItemClass::Chemicals => Some(ItemClass::Chemicals),
        IroseItemClass::Material => Some(ItemClass::Material),
        IroseItemClass::GatheredGoods => Some(ItemClass::GatheredGoods),
        IroseItemClass::Arrow => Some(ItemClass::Arrow),
        IroseItemClass::Bullet => Some(ItemClass::Bullet),
        IroseItemClass::Shell => Some(ItemClass::Shell),
        IroseItemClass::QuestItems => Some(ItemClass::QuestItems),
        IroseItemClass::Certification => Some(ItemClass::Certification),
        IroseItemClass::CartBody => Some(ItemClass::CartBody),
        IroseItemClass::CastleGearBody => Some(ItemClass::CastleGearBody),
        IroseItemClass::CartEngine => Some(ItemClass::CartEngine),
        IroseItemClass::CastleGearEngine => Some(ItemClass::CastleGearEngine),
        IroseItemClass::CartWheels => Some(ItemClass::CartWheels),
        IroseItemClass::CastleGearLeg => Some(ItemClass::CastleGearLeg),
        IroseItemClass::CartAccessory => Some(ItemClass::CartAccessory),
        IroseItemClass::CastleGearWeapon => Some(ItemClass::CastleGearWeapon),
    }
}

pub fn encode_item_class(item_class: ItemClass) -> Option<usize> {
    match item_class {
        ItemClass::Unknown => IroseItemClass::Unknown.to_usize(),
        ItemClass::FaceMask => IroseItemClass::FaceMask.to_usize(),
        ItemClass::FaceGlasses => IroseItemClass::FaceGlasses.to_usize(),
        ItemClass::FaceEtc => IroseItemClass::FaceEtc.to_usize(),
        ItemClass::Helmet => IroseItemClass::Helmet.to_usize(),
        ItemClass::MagicHat => IroseItemClass::MagicHat.to_usize(),
        ItemClass::Hat => IroseItemClass::Hat.to_usize(),
        ItemClass::HairAccessory => IroseItemClass::HairAccessory.to_usize(),
        ItemClass::CombatUniform => IroseItemClass::CombatUniform.to_usize(),
        ItemClass::MagicClothes => IroseItemClass::MagicClothes.to_usize(),
        ItemClass::CasualClothes => IroseItemClass::CasualClothes.to_usize(),
        ItemClass::Gauntlet => IroseItemClass::Gauntlet.to_usize(),
        ItemClass::MagicGlove => IroseItemClass::MagicGlove.to_usize(),
        ItemClass::Glove => IroseItemClass::Glove.to_usize(),
        ItemClass::Boots => IroseItemClass::Boots.to_usize(),
        ItemClass::MagicBoots => IroseItemClass::MagicBoots.to_usize(),
        ItemClass::Shoes => IroseItemClass::Shoes.to_usize(),
        ItemClass::BackArmor => IroseItemClass::BackArmor.to_usize(),
        ItemClass::Bag => IroseItemClass::Bag.to_usize(),
        ItemClass::Wings => IroseItemClass::Wings.to_usize(),
        ItemClass::ArrowBox => IroseItemClass::ArrowBox.to_usize(),
        ItemClass::BulletBox => IroseItemClass::BulletBox.to_usize(),
        ItemClass::ShellBox => IroseItemClass::ShellBox.to_usize(),
        ItemClass::Ring => IroseItemClass::Ring.to_usize(),
        ItemClass::Necklace => IroseItemClass::Necklace.to_usize(),
        ItemClass::Earring => IroseItemClass::Earring.to_usize(),
        ItemClass::OneHandedSword => IroseItemClass::OneHandedSword.to_usize(),
        ItemClass::OneHandedBlunt => IroseItemClass::OneHandedBlunt.to_usize(),
        ItemClass::TwoHandedSword => IroseItemClass::TwoHandedSword.to_usize(),
        ItemClass::Spear => IroseItemClass::Spear.to_usize(),
        ItemClass::TwoHandedAxe => IroseItemClass::TwoHandedAxe.to_usize(),
        ItemClass::Bow => IroseItemClass::Bow.to_usize(),
        ItemClass::Gun => IroseItemClass::Gun.to_usize(),
        ItemClass::Launcher => IroseItemClass::Launcher.to_usize(),
        ItemClass::MagicStaff => IroseItemClass::MagicStaff.to_usize(),
        ItemClass::MagicWand => IroseItemClass::MagicWand.to_usize(),
        ItemClass::Katar => IroseItemClass::Katar.to_usize(),
        ItemClass::DualSwords => IroseItemClass::DualSwords.to_usize(),
        ItemClass::DualGuns => IroseItemClass::DualGuns.to_usize(),
        ItemClass::Shield => IroseItemClass::Shield.to_usize(),
        ItemClass::SupportTool => IroseItemClass::SupportTool.to_usize(),
        ItemClass::Crossbow => IroseItemClass::Crossbow.to_usize(),
        ItemClass::Medicine => IroseItemClass::Medicine.to_usize(),
        ItemClass::Food => IroseItemClass::Food.to_usize(),
        ItemClass::MagicItem => IroseItemClass::MagicItem.to_usize(),
        ItemClass::SkillBook => IroseItemClass::SkillBook.to_usize(),
        ItemClass::RepairTool => IroseItemClass::RepairTool.to_usize(),
        ItemClass::QuestScroll => IroseItemClass::QuestScroll.to_usize(),
        ItemClass::EngineFuel => IroseItemClass::EngineFuel.to_usize(),
        ItemClass::AutomaticConsumption => IroseItemClass::AutomaticConsumption.to_usize(),
        ItemClass::TimeCoupon => IroseItemClass::TimeCoupon.to_usize(),
        ItemClass::Jewel => IroseItemClass::Jewel.to_usize(),
        ItemClass::WorkOfArt => IroseItemClass::WorkOfArt.to_usize(),
        ItemClass::Metal => IroseItemClass::Metal.to_usize(),
        ItemClass::OtherworldlyMetal => IroseItemClass::OtherworldlyMetal.to_usize(),
        ItemClass::StoneMaterial => IroseItemClass::StoneMaterial.to_usize(),
        ItemClass::WoodenMaterial => IroseItemClass::WoodenMaterial.to_usize(),
        ItemClass::Leather => IroseItemClass::Leather.to_usize(),
        ItemClass::Cloth => IroseItemClass::Cloth.to_usize(),
        ItemClass::RefiningMaterial => IroseItemClass::RefiningMaterial.to_usize(),
        ItemClass::Chemicals => IroseItemClass::Chemicals.to_usize(),
        ItemClass::Material => IroseItemClass::Material.to_usize(),
        ItemClass::GatheredGoods => IroseItemClass::GatheredGoods.to_usize(),
        ItemClass::Arrow => IroseItemClass::Arrow.to_usize(),
        ItemClass::Bullet => IroseItemClass::Bullet.to_usize(),
        ItemClass::Shell => IroseItemClass::Shell.to_usize(),
        ItemClass::QuestItems => IroseItemClass::QuestItems.to_usize(),
        ItemClass::Certification => IroseItemClass::Certification.to_usize(),
        ItemClass::CartBody => IroseItemClass::CartBody.to_usize(),
        ItemClass::CastleGearBody => IroseItemClass::CastleGearBody.to_usize(),
        ItemClass::CartEngine => IroseItemClass::CartEngine.to_usize(),
        ItemClass::CastleGearEngine => IroseItemClass::CastleGearEngine.to_usize(),
        ItemClass::CartWheels => IroseItemClass::CartWheels.to_usize(),
        ItemClass::CastleGearLeg => IroseItemClass::CastleGearLeg.to_usize(),
        ItemClass::CartAccessory => IroseItemClass::CartAccessory.to_usize(),
        ItemClass::CastleGearWeapon => IroseItemClass::CastleGearWeapon.to_usize(),
    }
}

pub fn decode_item_type(id: usize) -> Option<ItemType> {
    match FromPrimitive::from_usize(id)? {
        IroseItemType::Face => Some(ItemType::Face),
        IroseItemType::Head => Some(ItemType::Head),
        IroseItemType::Body => Some(ItemType::Body),
        IroseItemType::Hands => Some(ItemType::Hands),
        IroseItemType::Feet => Some(ItemType::Feet),
        IroseItemType::Back => Some(ItemType::Back),
        IroseItemType::Jewellery => Some(ItemType::Jewellery),
        IroseItemType::Weapon => Some(ItemType::Weapon),
        IroseItemType::SubWeapon => Some(ItemType::SubWeapon),
        IroseItemType::Consumable => Some(ItemType::Consumable),
        IroseItemType::Gem => Some(ItemType::Gem),
        IroseItemType::Material => Some(ItemType::Material),
        IroseItemType::Quest => Some(ItemType::Quest),
        IroseItemType::Vehicle => Some(ItemType::Vehicle),
    }
}

pub fn encode_item_type(id: ItemType) -> Option<usize> {
    match id {
        ItemType::Face => IroseItemType::Face.to_usize(),
        ItemType::Head => IroseItemType::Head.to_usize(),
        ItemType::Body => IroseItemType::Body.to_usize(),
        ItemType::Hands => IroseItemType::Hands.to_usize(),
        ItemType::Feet => IroseItemType::Feet.to_usize(),
        ItemType::Back => IroseItemType::Back.to_usize(),
        ItemType::Jewellery => IroseItemType::Jewellery.to_usize(),
        ItemType::Weapon => IroseItemType::Weapon.to_usize(),
        ItemType::SubWeapon => IroseItemType::SubWeapon.to_usize(),
        ItemType::Consumable => IroseItemType::Consumable.to_usize(),
        ItemType::Gem => IroseItemType::Gem.to_usize(),
        ItemType::Material => IroseItemType::Material.to_usize(),
        ItemType::Quest => IroseItemType::Quest.to_usize(),
        ItemType::Vehicle => IroseItemType::Vehicle.to_usize(),
    }
}

pub fn decode_equipment_index(id: usize) -> Option<EquipmentIndex> {
    match FromPrimitive::from_usize(id)? {
        IroseEquipmentIndex::Face => Some(EquipmentIndex::Face),
        IroseEquipmentIndex::Head => Some(EquipmentIndex::Head),
        IroseEquipmentIndex::Body => Some(EquipmentIndex::Body),
        IroseEquipmentIndex::Back => Some(EquipmentIndex::Back),
        IroseEquipmentIndex::Hands => Some(EquipmentIndex::Hands),
        IroseEquipmentIndex::Feet => Some(EquipmentIndex::Feet),
        IroseEquipmentIndex::Weapon => Some(EquipmentIndex::Weapon),
        IroseEquipmentIndex::SubWeapon => Some(EquipmentIndex::SubWeapon),
        IroseEquipmentIndex::Necklace => Some(EquipmentIndex::Necklace),
        IroseEquipmentIndex::Ring => Some(EquipmentIndex::Ring),
        IroseEquipmentIndex::Earring => Some(EquipmentIndex::Earring),
    }
}

pub fn decode_vehicle_part_index(id: usize) -> Option<VehiclePartIndex> {
    match FromPrimitive::from_usize(id)? {
        IroseVehiclePartIndex::Body => Some(VehiclePartIndex::Body),
        IroseVehiclePartIndex::Engine => Some(VehiclePartIndex::Engine),
        IroseVehiclePartIndex::Leg => Some(VehiclePartIndex::Leg),
        IroseVehiclePartIndex::Arms => Some(VehiclePartIndex::Arms),
    }
}

pub fn decode_vehicle_type(id: usize) -> Option<VehicleType> {
    match FromPrimitive::from_usize(id)? {
        IroseVehicleType::Cart => Some(VehicleType::Cart),
        IroseVehicleType::CastleGear => Some(VehicleType::CastleGear),
    }
}

pub fn decode_ammo_index(id: usize) -> Option<AmmoIndex> {
    match FromPrimitive::from_usize(id)? {
        IroseAmmoIndex::Arrow => Some(AmmoIndex::Arrow),
        IroseAmmoIndex::Bullet => Some(AmmoIndex::Bullet),
        IroseAmmoIndex::Throw => Some(AmmoIndex::Throw),
    }
}

pub fn encode_equipment_index(id: EquipmentIndex) -> Option<usize> {
    match id {
        EquipmentIndex::Face => IroseEquipmentIndex::Face.to_usize(),
        EquipmentIndex::Head => IroseEquipmentIndex::Head.to_usize(),
        EquipmentIndex::Body => IroseEquipmentIndex::Body.to_usize(),
        EquipmentIndex::Back => IroseEquipmentIndex::Back.to_usize(),
        EquipmentIndex::Hands => IroseEquipmentIndex::Hands.to_usize(),
        EquipmentIndex::Feet => IroseEquipmentIndex::Feet.to_usize(),
        EquipmentIndex::Weapon => IroseEquipmentIndex::Weapon.to_usize(),
        EquipmentIndex::SubWeapon => IroseEquipmentIndex::SubWeapon.to_usize(),
        EquipmentIndex::Necklace => IroseEquipmentIndex::Necklace.to_usize(),
        EquipmentIndex::Ring => IroseEquipmentIndex::Ring.to_usize(),
        EquipmentIndex::Earring => IroseEquipmentIndex::Earring.to_usize(),
    }
}

pub fn encode_vehicle_part_index(id: VehiclePartIndex) -> Option<usize> {
    match id {
        VehiclePartIndex::Body => IroseVehiclePartIndex::Body.to_usize(),
        VehiclePartIndex::Engine => IroseVehiclePartIndex::Engine.to_usize(),
        VehiclePartIndex::Leg => IroseVehiclePartIndex::Leg.to_usize(),
        VehiclePartIndex::Arms => IroseVehiclePartIndex::Arms.to_usize(),
    }
}

pub fn encode_ammo_index(id: AmmoIndex) -> Option<usize> {
    match id {
        AmmoIndex::Arrow => IroseAmmoIndex::Arrow.to_usize(),
        AmmoIndex::Bullet => IroseAmmoIndex::Bullet.to_usize(),
        AmmoIndex::Throw => IroseAmmoIndex::Throw.to_usize(),
    }
}

pub fn decode_status_effect_type(id: usize) -> Option<StatusEffectType> {
    match FromPrimitive::from_usize(id)? {
        IroseStatusEffectType::IncreaseHp => Some(StatusEffectType::IncreaseHp),
        IroseStatusEffectType::IncreaseMp => Some(StatusEffectType::IncreaseMp),
        IroseStatusEffectType::Poisoned => Some(StatusEffectType::Poisoned),
        IroseStatusEffectType::IncreaseMaxHp => Some(StatusEffectType::IncreaseMaxHp),
        IroseStatusEffectType::IncreaseMaxMp => Some(StatusEffectType::IncreaseMaxMp),
        IroseStatusEffectType::IncreaseMoveSpeed => Some(StatusEffectType::IncreaseMoveSpeed),
        IroseStatusEffectType::DecreaseMoveSpeed => Some(StatusEffectType::DecreaseMoveSpeed),
        IroseStatusEffectType::IncreaseAttackSpeed => Some(StatusEffectType::IncreaseAttackSpeed),
        IroseStatusEffectType::DecreaseAttackSpeed => Some(StatusEffectType::DecreaseAttackSpeed),
        IroseStatusEffectType::IncreaseAttackPower => Some(StatusEffectType::IncreaseAttackPower),
        IroseStatusEffectType::DecreaseAttackPower => Some(StatusEffectType::DecreaseAttackPower),
        IroseStatusEffectType::IncreaseDefence => Some(StatusEffectType::IncreaseDefence),
        IroseStatusEffectType::DecreaseDefence => Some(StatusEffectType::DecreaseDefence),
        IroseStatusEffectType::IncreaseResistance => Some(StatusEffectType::IncreaseResistance),
        IroseStatusEffectType::DecreaseResistance => Some(StatusEffectType::DecreaseResistance),
        IroseStatusEffectType::IncreaseHit => Some(StatusEffectType::IncreaseHit),
        IroseStatusEffectType::DecreaseHit => Some(StatusEffectType::DecreaseHit),
        IroseStatusEffectType::IncreaseCritical => Some(StatusEffectType::IncreaseCritical),
        IroseStatusEffectType::DecreaseCritical => Some(StatusEffectType::DecreaseCritical),
        IroseStatusEffectType::IncreaseAvoid => Some(StatusEffectType::IncreaseAvoid),
        IroseStatusEffectType::DecreaseAvoid => Some(StatusEffectType::DecreaseAvoid),
        IroseStatusEffectType::Dumb => Some(StatusEffectType::Dumb),
        IroseStatusEffectType::Sleep => Some(StatusEffectType::Sleep),
        IroseStatusEffectType::Fainting => Some(StatusEffectType::Fainting),
        IroseStatusEffectType::Disguise => Some(StatusEffectType::Disguise),
        IroseStatusEffectType::Transparent => Some(StatusEffectType::Transparent),
        IroseStatusEffectType::ShieldDamage => Some(StatusEffectType::ShieldDamage),
        IroseStatusEffectType::AdditionalDamageRate => Some(StatusEffectType::AdditionalDamageRate),
        IroseStatusEffectType::DecreaseLifeTime => Some(StatusEffectType::DecreaseLifeTime),
        IroseStatusEffectType::ClearGood => Some(StatusEffectType::ClearGood),
        IroseStatusEffectType::ClearBad => Some(StatusEffectType::ClearBad),
        IroseStatusEffectType::ClearAll => Some(StatusEffectType::ClearAll),
        IroseStatusEffectType::ClearInvisible => Some(StatusEffectType::ClearInvisible),
        IroseStatusEffectType::Taunt => Some(StatusEffectType::Taunt),
        IroseStatusEffectType::Revive => Some(StatusEffectType::Revive),
    }
}

pub fn decode_status_effect_cleared_by_type(id: usize) -> Option<StatusEffectClearedByType> {
    match FromPrimitive::from_usize(id)? {
        IroseStatusEffectClearedByType::Good => Some(StatusEffectClearedByType::ClearGood),
        IroseStatusEffectClearedByType::Bad => Some(StatusEffectClearedByType::ClearBad),
        IroseStatusEffectClearedByType::None => Some(StatusEffectClearedByType::ClearNone),
    }
}

pub fn decode_skill_action_mode(id: usize) -> Option<SkillActionMode> {
    match FromPrimitive::from_usize(id)? {
        IroseSkillActionMode::Stop => Some(SkillActionMode::Stop),
        IroseSkillActionMode::Attack => Some(SkillActionMode::Attack),
        IroseSkillActionMode::Restore => Some(SkillActionMode::Restore),
    }
}

pub fn decode_skill_basic_command(id: usize) -> Option<SkillBasicCommand> {
    match FromPrimitive::from_usize(id)? {
        IroseSkillBasicCommand::Sit => Some(SkillBasicCommand::Sit),
        IroseSkillBasicCommand::PickupItem => Some(SkillBasicCommand::PickupItem),
        IroseSkillBasicCommand::Jump => Some(SkillBasicCommand::Jump),
        IroseSkillBasicCommand::AirJump => Some(SkillBasicCommand::AirJump),
        IroseSkillBasicCommand::AutoTarget => Some(SkillBasicCommand::AutoTarget),
        IroseSkillBasicCommand::Attack => Some(SkillBasicCommand::Attack),
        IroseSkillBasicCommand::DriveVehicle => Some(SkillBasicCommand::DriveVehicle),
        IroseSkillBasicCommand::AddFriend => Some(SkillBasicCommand::AddFriend),
        IroseSkillBasicCommand::PartyInvite => Some(SkillBasicCommand::PartyInvite),
        IroseSkillBasicCommand::Trade => Some(SkillBasicCommand::Trade),
        IroseSkillBasicCommand::PrivateStore => Some(SkillBasicCommand::PrivateStore),
        IroseSkillBasicCommand::SelfTarget => Some(SkillBasicCommand::SelfTarget),
        IroseSkillBasicCommand::VehiclePassengerInvite => {
            Some(SkillBasicCommand::VehiclePassengerInvite)
        }
    }
}

pub fn decode_skill_target_filter(id: usize) -> Option<SkillTargetFilter> {
    match FromPrimitive::from_usize(id)? {
        IroseSkillTargetFilter::OnlySelf => Some(SkillTargetFilter::OnlySelf),
        IroseSkillTargetFilter::Group => Some(SkillTargetFilter::Group),
        IroseSkillTargetFilter::Guild => Some(SkillTargetFilter::Guild),
        IroseSkillTargetFilter::Allied => Some(SkillTargetFilter::Allied),
        IroseSkillTargetFilter::Monster => Some(SkillTargetFilter::Monster),
        IroseSkillTargetFilter::Enemy => Some(SkillTargetFilter::Enemy),
        IroseSkillTargetFilter::EnemyCharacter => Some(SkillTargetFilter::EnemyCharacter),
        IroseSkillTargetFilter::Character => Some(SkillTargetFilter::Character),
        IroseSkillTargetFilter::CharacterOrMonster => Some(SkillTargetFilter::CharacterOrMonster),
        IroseSkillTargetFilter::DeadAlliedCharacter => Some(SkillTargetFilter::DeadAlliedCharacter),
        IroseSkillTargetFilter::EnemyMonster => Some(SkillTargetFilter::EnemyMonster),
    }
}

pub fn encode_skill_target_filter(skill_target_filter: SkillTargetFilter) -> Option<usize> {
    match skill_target_filter {
        SkillTargetFilter::OnlySelf => IroseSkillTargetFilter::OnlySelf.to_usize(),
        SkillTargetFilter::Group => IroseSkillTargetFilter::Group.to_usize(),
        SkillTargetFilter::Guild => IroseSkillTargetFilter::Guild.to_usize(),
        SkillTargetFilter::Allied => IroseSkillTargetFilter::Allied.to_usize(),
        SkillTargetFilter::Monster => IroseSkillTargetFilter::Monster.to_usize(),
        SkillTargetFilter::Enemy => IroseSkillTargetFilter::Enemy.to_usize(),
        SkillTargetFilter::EnemyCharacter => IroseSkillTargetFilter::EnemyCharacter.to_usize(),
        SkillTargetFilter::Character => IroseSkillTargetFilter::Character.to_usize(),
        SkillTargetFilter::CharacterOrMonster => {
            IroseSkillTargetFilter::CharacterOrMonster.to_usize()
        }
        SkillTargetFilter::DeadAlliedCharacter => {
            IroseSkillTargetFilter::DeadAlliedCharacter.to_usize()
        }
        SkillTargetFilter::EnemyMonster => IroseSkillTargetFilter::EnemyMonster.to_usize(),
    }
}

pub fn decode_skill_type(id: usize) -> Option<SkillType> {
    match FromPrimitive::from_usize(id)? {
        IroseSkillType::BasicAction => Some(SkillType::BasicAction),
        IroseSkillType::CreateWindow => Some(SkillType::CreateWindow),
        IroseSkillType::Immediate => Some(SkillType::Immediate),
        IroseSkillType::EnforceWeapon => Some(SkillType::EnforceWeapon),
        IroseSkillType::EnforceBullet => Some(SkillType::EnforceBullet),
        IroseSkillType::FireBullet => Some(SkillType::FireBullet),
        IroseSkillType::AreaTarget => Some(SkillType::AreaTarget),
        IroseSkillType::SelfBoundDuration => Some(SkillType::SelfBoundDuration),
        IroseSkillType::TargetBoundDuration => Some(SkillType::TargetBoundDuration),
        IroseSkillType::SelfBound => Some(SkillType::SelfBound),
        IroseSkillType::TargetBound => Some(SkillType::TargetBound),
        IroseSkillType::SelfStateDuration => Some(SkillType::SelfStateDuration),
        IroseSkillType::TargetStateDuration => Some(SkillType::TargetStateDuration),
        IroseSkillType::SummonPet => Some(SkillType::SummonPet),
        IroseSkillType::Passive => Some(SkillType::Passive),
        IroseSkillType::Emote => Some(SkillType::Emote),
        IroseSkillType::SelfDamage => Some(SkillType::SelfDamage),
        IroseSkillType::Warp => Some(SkillType::Warp),
        IroseSkillType::SelfAndTarget => Some(SkillType::SelfAndTarget),
        IroseSkillType::Resurrection => Some(SkillType::Resurrection),
    }
}

pub fn encode_skill_type(skill_type: SkillType) -> Option<usize> {
    match skill_type {
        SkillType::BasicAction => IroseSkillType::BasicAction.to_usize(),
        SkillType::CreateWindow => IroseSkillType::CreateWindow.to_usize(),
        SkillType::Immediate => IroseSkillType::Immediate.to_usize(),
        SkillType::EnforceWeapon => IroseSkillType::EnforceWeapon.to_usize(),
        SkillType::EnforceBullet => IroseSkillType::EnforceBullet.to_usize(),
        SkillType::FireBullet => IroseSkillType::FireBullet.to_usize(),
        SkillType::AreaTarget => IroseSkillType::AreaTarget.to_usize(),
        SkillType::SelfBoundDuration => IroseSkillType::SelfBoundDuration.to_usize(),
        SkillType::TargetBoundDuration => IroseSkillType::TargetBoundDuration.to_usize(),
        SkillType::SelfBound => IroseSkillType::SelfBound.to_usize(),
        SkillType::TargetBound => IroseSkillType::TargetBound.to_usize(),
        SkillType::SelfStateDuration => IroseSkillType::SelfStateDuration.to_usize(),
        SkillType::TargetStateDuration => IroseSkillType::TargetStateDuration.to_usize(),
        SkillType::SummonPet => IroseSkillType::SummonPet.to_usize(),
        SkillType::Passive => IroseSkillType::Passive.to_usize(),
        SkillType::Emote => IroseSkillType::Emote.to_usize(),
        SkillType::SelfDamage => IroseSkillType::SelfDamage.to_usize(),
        SkillType::Warp => IroseSkillType::Warp.to_usize(),
        SkillType::SelfAndTarget => IroseSkillType::SelfAndTarget.to_usize(),
        SkillType::Resurrection => IroseSkillType::Resurrection.to_usize(),
    }
}

pub fn decode_clan_member_position(id: usize) -> Option<ClanMemberPosition> {
    match FromPrimitive::from_usize(id)? {
        IroseClanMemberPosition::Penalty => Some(ClanMemberPosition::Penalty),
        IroseClanMemberPosition::Junior => Some(ClanMemberPosition::Junior),
        IroseClanMemberPosition::Senior => Some(ClanMemberPosition::Senior),
        IroseClanMemberPosition::Veteran => Some(ClanMemberPosition::Veteran),
        IroseClanMemberPosition::Commander => Some(ClanMemberPosition::Commander),
        IroseClanMemberPosition::DeputyMaster => Some(ClanMemberPosition::DeputyMaster),
        IroseClanMemberPosition::Master => Some(ClanMemberPosition::Master),
    }
}

pub fn encode_clan_member_position(position: ClanMemberPosition) -> Option<usize> {
    match position {
        ClanMemberPosition::Penalty => IroseClanMemberPosition::Penalty.to_usize(),
        ClanMemberPosition::Junior => IroseClanMemberPosition::Junior.to_usize(),
        ClanMemberPosition::Senior => IroseClanMemberPosition::Senior.to_usize(),
        ClanMemberPosition::Veteran => IroseClanMemberPosition::Veteran.to_usize(),
        ClanMemberPosition::Commander => IroseClanMemberPosition::Commander.to_usize(),
        ClanMemberPosition::DeputyMaster => IroseClanMemberPosition::DeputyMaster.to_usize(),
        ClanMemberPosition::Master => IroseClanMemberPosition::Master.to_usize(),
    }
}
