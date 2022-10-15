use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::{
    num::{NonZeroU16, NonZeroUsize},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use crate::{
    effect_database::EffectId, AbilityType, EffectFileId, ItemClass, JobClassId, MotionId, NpcId,
    SoundId, StatusEffectId, StringDatabase, ZoneId,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug)]
pub struct SkillCooldownGroup(pub NonZeroUsize);

#[derive(Copy, Clone, Debug)]
pub enum SkillCooldown {
    Skill(Duration),
    Group(SkillCooldownGroup, Duration),
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
    pub damage_type: i32,
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
