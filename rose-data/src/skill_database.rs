use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::{NonZeroU16, NonZeroUsize},
    str::FromStr,
    time::Duration,
};

use crate::{AbilityType, ItemClass, MotionId, NpcId, StatusEffectId, ZoneId};

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct SkillId(NonZeroU16);

id_wrapper_impl!(SkillId, NonZeroU16, u16);

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum SkillPageType {
    Basic,
    Active,
    Passive,
    Clan,
}

#[derive(Debug)]
pub struct SkillAddAbility {
    pub ability_type: AbilityType,
    pub rate: i32,
    pub value: i32,
}

#[derive(Debug)]
pub enum SkillActionMode {
    Stop,
    Attack,
    Restore,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SkillCooldownGroup(pub NonZeroUsize);

#[derive(Debug)]
pub enum SkillCooldown {
    Skill(Duration),
    Group(SkillCooldownGroup, Duration),
}

// TODO: Make SkillData an enum on SkillType with relevant fields only?
#[derive(Debug)]
pub struct SkillData {
    pub id: SkillId,
    pub name: String,

    pub base_skill_id: Option<SkillId>,
    pub level: u32,
    pub learn_point_cost: u32,
    pub learn_money_cost: u32,
    pub skill_type: SkillType,
    pub page: SkillPageType,
    pub icon_number: u32,

    pub use_ability: ArrayVec<(AbilityType, i32), 2>,
    pub required_ability: ArrayVec<(AbilityType, i32), 2>,
    pub required_job_set_index: Option<NonZeroUsize>, // TODO: JobSetReference to the job set STB
    pub required_planet: Option<NonZeroUsize>,
    pub required_skills: ArrayVec<(SkillId, i32), 3>,
    pub required_union: ArrayVec<NonZeroUsize, 3>,
    pub required_weapon_class: ArrayVec<ItemClass, 5>,

    pub action_mode: SkillActionMode,
    pub action_motion_id: Option<MotionId>,
    pub action_motion_speed: f32,
    pub add_ability: [Option<SkillAddAbility>; 2],
    pub cast_range: u32,
    pub casting_motion_id: Option<MotionId>,
    pub casting_motion_speed: f32,
    pub casting_repeat_motion_id: Option<MotionId>,
    pub casting_repeat_motion_count: u32,
    pub cooldown: SkillCooldown,
    pub damage_type: i32,
    pub harm: u32,
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
    skills: HashMap<u16, SkillData>,
}

impl SkillDatabase {
    pub fn new(skills: HashMap<u16, SkillData>) -> Self {
        Self { skills }
    }

    pub fn get_skill(&self, id: SkillId) -> Option<&SkillData> {
        self.skills.get(&id.get())
    }
}
