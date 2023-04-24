mod bot_attack_nearby;
mod bot_attack_target;
mod bot_attack_threat;
mod bot_find_monster_spawn;
mod bot_join_zone;
mod bot_pickup_item;
mod bot_revive;
mod bot_sit_recover_hp;
mod bot_snowball_fight;
mod bot_use_attack_skill;

mod create_bot;

const IDLE_DURATION: Duration = Duration::from_millis(250);

use std::time::Duration;

use bevy::prelude::{Component, Entity, IntoSystemConfigs, Plugin};
use big_brain::{
    prelude::Highest,
    thinker::{Thinker, ThinkerBuilder},
    BigBrainPlugin, BigBrainSet,
};

pub use create_bot::create_bot;

use bot_attack_nearby::{
    action_attack_random_nearby_target, score_find_nearby_target, AttackRandomNearbyTarget,
    FindNearbyTarget,
};
use bot_attack_target::{
    action_attack_target, score_should_attack_target, ActionAttackTarget, ShouldAttackTarget,
};
use bot_attack_threat::{
    action_attack_threat, score_threat_is_not_target, AttackThreat, ThreatIsNotTarget,
};
use bot_find_monster_spawn::{action_find_monster_spawn, FindMonsterSpawns};
use bot_join_zone::{action_join_zone, score_is_teleporting, IsTeleporting, JoinZone};
use bot_pickup_item::{
    action_pickup_nearest_item_drop, score_find_nearby_item_drop_system, FindNearbyItemDrop,
    PickupNearestItemDrop,
};
use bot_revive::{action_revive_current_zone, score_is_dead, IsDead, ReviveCurrentZone};
use bot_sit_recover_hp::{
    action_sit_recover_hp, score_should_sit_recover_hp, ShouldSitRecoverHp, SitRecoverHp,
};
use bot_snowball_fight::{action_snowball_fight, SnowballFight};
use bot_use_attack_skill::{
    action_use_attack_skill, score_should_use_attack_skill, ShouldUseAttackSkill, UseAttackSkill,
};

#[derive(Component)]
pub struct BotCombatTarget {
    entity: Entity,
}

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BigBrainPlugin)
            .add_systems(
                (
                    action_attack_random_nearby_target,
                    action_attack_threat,
                    action_pickup_nearest_item_drop,
                    action_snowball_fight,
                    action_attack_target,
                    action_revive_current_zone,
                    action_join_zone,
                    action_sit_recover_hp,
                    action_find_monster_spawn,
                    action_use_attack_skill,
                )
                    .in_set(BigBrainSet::Actions),
            )
            .add_systems(
                (
                    score_find_nearby_target,
                    score_threat_is_not_target,
                    score_find_nearby_item_drop_system,
                    score_should_attack_target,
                    score_is_dead,
                    score_is_teleporting,
                    score_should_sit_recover_hp,
                    score_should_use_attack_skill,
                )
                    .in_set(BigBrainSet::Scorers),
            );
    }
}

pub fn bot_thinker() -> ThinkerBuilder {
    Thinker::build()
        .picker(Highest)
        .when(IsDead { score: 1.0 }, ReviveCurrentZone)
        .when(IsTeleporting { score: 1.0 }, JoinZone)
        .when(ThreatIsNotTarget { score: 1.0 }, AttackThreat)
        .when(ShouldUseAttackSkill { score: 0.9 }, UseAttackSkill)
        .when(
            ShouldAttackTarget {
                min_score: 0.6,
                max_score: 0.8,
            },
            ActionAttackTarget,
        )
        .when(FindNearbyItemDrop { score: 0.5 }, PickupNearestItemDrop)
        .when(ShouldSitRecoverHp { score: 0.4 }, SitRecoverHp)
        .when(FindNearbyTarget { score: 0.2 }, AttackRandomNearbyTarget)
        .otherwise(FindMonsterSpawns)
}

pub fn bot_snowball_fight() -> ThinkerBuilder {
    Thinker::build()
        .picker(Highest)
        .otherwise(SnowballFight::default())
}
