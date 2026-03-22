use crate::{
    effect_database::EffectId, AbilityType, EffectFileId, ItemClass, JobClassId, MotionId, NpcId,
    SoundId, StatusEffectId, StringDatabase, ZoneId,
};
use arrayvec::ArrayVec;
use bevy::reflect::Reflect;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

pub enum SkillIds {
    Sit = 11,
    PickUp = 12,
    Jump = 13,
    Attack = 16,
    Party = 19,
    Trade = 20,
    Vending = 21,
    RideRequest = 25,
    Hi = 41,
    Bow = 42,
    Salutation = 43,
    RecoveryKiss = 44,
    CharmingKiss = 45,
    Laugh = 46,
    FightCheer = 47,
    BreakDown = 48,
    Tantrum = 49,
    Applause = 50,
    MeleeWeaponMastery = 201,
    OneHandedMastery = 211,
    TwoHandedMastery = 221,
    PhysicalTraining = 231,
    DefenceTraining = 251,
    QuickStep = 281,
    SpiritualTraining = 291,
    HeavyAttack = 301,
    DoubleAttack = 321,
    LeapAttack = 341,
    DivineForce = 391,
    Taunt = 411,
    LightningCrusher = 521,
    SpaceAttack = 661,
    ChampionHit = 671,
    StaffMastery = 801,
    Meditation = 821,
    SpellMastery = 841,
    ManaBolt = 901,
    FireRing = 911,
    Cure = 931,
    StaffStun = 941,
    Lightning = 951,
    LesserHaste = 961,
    IceBolt = 981,
    PowerSupport = 1021,
    Weaken = 1031,
    FireBurn = 1061,
    Silence = 1071,
    Resurrection = 1131,
    Bonfire = 1161,
    PhantomSword = 1171,
    CallFiregon = 1191,
    HitSupport = 1221,
    CriticalSupport = 1231,
    BowMastery = 1401,
    KnuckleMastery = 1421,
    Relax = 1441,
    HawkerSpirit = 1451,
    CombatMastery = 1461,
    AimShot = 1481,
    DoubleShot = 1521,
    DoubleSlash = 1541,
    TrapShot = 1581,
    ScrewAttack = 1591,
    HeartHit = 1601,
    PrimeHit = 1611,
    PoisonArrow = 1621,
    FlameHawk = 1641,
    Detect = 1671,
    HawkShot = 1701,
    ManaBlood = 1831,
    Stealth = 1841,
    PoisonKnife = 1851,
    BloodyDust = 1861,
    MagicKnife = 1871,
    MarketResearch = 2001,
    Marksmanship = 2021,
    BagpackMastery = 2041,
    Discount = 2051,
    Overcharge = 2071,
    Stockpile = 2091,
    WeaponResearch = 2111,
    ArmorResearch = 2131,
    ArmsMastery = 2141,
    MightyShot = 2201,
    SnipingShot = 2211,
    TwinBullets = 2221,
    ItemDisassembly = 2621,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Reflect)]
pub struct SkillId(NonZeroU16);

id_wrapper_impl!(SkillId, NonZeroU16, u16);

pub type SkillPageType = usize;

#[derive(Debug)]
pub struct SkillAddAbility {
    pub ability_type: AbilityType,
    pub rate: i32,
    pub value: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum SkillActionMode {
    Stop,
    Attack,
    Restore,
}

#[derive(Copy, Clone, Debug)]
pub enum SkillTargetFilter {
    OnlySelf,
    Group,
    Guild,
    Allied,
    Monster,
    Enemy,
    EnemyCharacter,
    Character,
    CharacterOrMonster,
    DeadAlliedCharacter,
    EnemyMonster,
}

#[derive(Copy, Clone, Debug)]
pub enum SkillType {
    BasicAction,
    CreateWindow,
    Immediate,
    EnforceWeapon,
    EnforceBullet,
    FireBullet,
    AreaTarget,
    SelfBoundDuration,
    TargetBoundDuration,
    SelfBound,
    TargetBound,
    SelfStateDuration,
    TargetStateDuration,
    SummonPet,
    Passive,
    Emote,
    SelfDamage,
    Warp,
    SelfAndTarget,
    Resurrection,
}

#[derive(Copy, Clone, Debug)]
pub enum SkillBasicCommand {
    Sit,
    PickupItem,
    Jump,
    AirJump,
    AutoTarget,
    Attack,
    DriveVehicle,
    AddFriend,
    PartyInvite,
    Trade,
    PrivateStore,
    SelfTarget,
    VehiclePassengerInvite,
}

impl SkillType {
    pub fn is_self_skill(&self) -> bool {
        matches!(
            self,
            SkillType::SelfBoundDuration
                | SkillType::SelfBound
                | SkillType::SelfStateDuration
                | SkillType::SummonPet
                | SkillType::SelfDamage
        )
    }

    pub fn is_target_skill(&self) -> bool {
        matches!(
            self,
            SkillType::Immediate
                | SkillType::EnforceWeapon
                | SkillType::EnforceBullet
                | SkillType::FireBullet
                | SkillType::TargetBoundDuration
                | SkillType::TargetBound
                | SkillType::TargetStateDuration
                | SkillType::SelfAndTarget
                | SkillType::Resurrection
        )
    }
}

pub type SkillCooldownGroup = NonZeroUsize;

#[derive(Copy, Clone, Debug)]
pub enum SkillCooldown {
    Skill {
        duration: Duration,
    },
    Group {
        group: SkillCooldownGroup,
        duration: Duration,
    },
}

#[derive(FromPrimitive, Debug)]
pub enum SkillDamageType {
    ContinuousAttack = 0,
    WeaponAttack = 1,
    MagicAttack = 2,
    NaturalMagic = 3,
}

#[derive(Copy, Clone, Debug)]
pub struct SkillCastingEffect {
    pub effect_file_id: EffectFileId,
    pub effect_dummy_bone_id: Option<usize>,
}

// TODO: Make SkillData an enum on SkillType with relevant fields only?
#[derive(Debug)]
pub struct SkillData {
    pub id: SkillId,
    pub name: &'static str,
    pub description: &'static str,

    pub base_skill_id: Option<SkillId>,
    pub level: u32,
    pub learn_point_cost: u32,
    pub learn_money_cost: u32,
    pub skill_type: SkillType,
    pub page: SkillPageType,
    pub icon_number: u32,

    pub use_ability: ArrayVec<(AbilityType, i32), 2>,
    pub required_ability: ArrayVec<(AbilityType, i32), 2>,
    pub required_job_class: Option<JobClassId>,
    pub required_planet: Option<NonZeroUsize>,
    pub required_skills: ArrayVec<(SkillId, i32), 3>,
    pub required_union: ArrayVec<NonZeroUsize, 3>,
    pub required_equipment_class: ArrayVec<ItemClass, 5>,

    pub action_mode: SkillActionMode,
    pub action_motion_id: Option<MotionId>,
    pub action_motion_speed: f32,
    pub add_ability: [Option<SkillAddAbility>; 2],
    pub basic_command: Option<SkillBasicCommand>,
    pub bullet_effect_id: Option<EffectId>,
    pub bullet_link_dummy_bone_id: u32,
    pub bullet_fire_sound_id: Option<SoundId>,
    pub cast_range: u32,
    pub casting_motion_id: Option<MotionId>,
    pub casting_motion_speed: f32,
    pub casting_repeat_motion_id: Option<MotionId>,
    pub casting_repeat_motion_count: u32,
    pub casting_effects: [Option<SkillCastingEffect>; 4],
    pub cooldown: SkillCooldown,
    pub damage_type: SkillDamageType,
    pub harm: u32,
    pub hit_effect_file_id: Option<EffectFileId>,
    pub hit_link_dummy_bone_id: Option<usize>,
    pub hit_sound_id: Option<SoundId>,
    pub hit_dummy_effect_file_id: [Option<EffectFileId>; 2],
    pub hit_dummy_sound_id: [Option<SoundId>; 2],
    pub item_make_number: u32,
    pub power: u32,
    pub scope: u32,
    pub status_effects: [Option<StatusEffectId>; 2],
    pub status_effect_duration: Duration,
    pub success_ratio: i32,
    pub summon_npc_id: Option<NpcId>,
    pub target_filter: SkillTargetFilter,
    pub warp_zone_id: Option<ZoneId>,
    pub warp_zone_x: f32,
    pub warp_zone_y: f32,
}

pub struct SkillDatabase {
    _string_database: Arc<StringDatabase>,
    skills: Vec<Option<SkillData>>,
}

impl SkillDatabase {
    pub fn new(string_database: Arc<StringDatabase>, skills: Vec<Option<SkillData>>) -> Self {
        Self {
            _string_database: string_database,
            skills,
        }
    }

    pub fn get_skill(&self, id: SkillId) -> Option<&SkillData> {
        self.skills.get(id.get() as usize).and_then(|x| x.as_ref())
    }

    pub fn iter(&self) -> impl Iterator<Item = &SkillData> {
        self.skills.iter().filter_map(|x| x.as_ref())
    }
}
