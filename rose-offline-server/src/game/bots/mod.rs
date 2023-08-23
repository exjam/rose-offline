mod bot_accept_party_invite;
mod bot_attack_target;
mod bot_attack_threat;
mod bot_find_monster_spawn;
mod bot_find_nearby_target;
mod bot_join_zone;
mod bot_pickup_item;
mod bot_revive;
mod bot_send_party_invite;
mod bot_sit_recover_hp;
mod bot_snowball_fight;
mod bot_use_attack_skill;
mod bot_use_buff_skill;

mod create_bot;

pub use create_bot::{
    bot_build_artisan, bot_build_bourgeois, bot_build_champion, bot_build_cleric, bot_build_knight,
    bot_build_mage, bot_build_raider, bot_build_scout, bot_create_random_build,
    bot_create_with_build, BotBuild,
};

use bot_accept_party_invite::{
    action_accept_party_invite, score_has_party_invite, AcceptPartyInvite, HasPartyInvite,
};
use bot_attack_target::{
    action_attack_target, score_should_attack_target, ActionAttackTarget, ShouldAttackTarget,
};
use bot_attack_threat::{
    action_attack_threat, score_threat_is_not_target, AttackThreat, ThreatIsNotTarget,
};
use bot_find_monster_spawn::{action_find_monster_spawn, FindMonsterSpawns};
use bot_find_nearby_target::{
    action_attack_random_nearby_target, score_find_nearby_target, AttackRandomNearbyTarget,
    FindNearbyTarget,
};
use bot_join_zone::{action_join_zone, score_is_teleporting, IsTeleporting, JoinZone};
use bot_pickup_item::{
    action_pickup_nearest_item_drop, score_find_nearby_item_drop_system, FindNearbyItemDrop,
    PickupNearestItemDrop,
};
use bot_revive::{action_revive_current_zone, score_is_dead, IsDead, ReviveCurrentZone};
use bot_send_party_invite::{
    action_party_invite_nearby_bot, score_can_party_invite_nearby_bot, CanPartyInviteNearbyBot,
    PartyInviteNearbyBot,
};
use bot_sit_recover_hp::{
    action_sit_recover_hp, score_should_sit_recover_hp, ShouldSitRecoverHp, SitRecoverHp,
};
use bot_snowball_fight::{action_snowball_fight, SnowballFight};
use bot_use_attack_skill::{
    action_use_attack_skill, score_should_use_attack_skill, ShouldUseAttackSkill, UseAttackSkill,
};
use bot_use_buff_skill::{
    action_use_buff_skill, score_should_use_buff_skill, ShouldUseBuffSkill, UseBuffSkill,
};

use bevy::prelude::{Component, Entity, IntoSystemConfigs, Plugin, PreUpdate, With, Without};
use big_brain::{
    prelude::Highest,
    thinker::{Thinker, ThinkerBuilder},
    BigBrainPlugin, BigBrainSet,
};
use std::time::Duration;

use crate::game::components::{ClientEntity, Dead};

const IDLE_DURATION: Duration = Duration::from_millis(250);

type BotQueryFilterAlive = (With<ClientEntity>, Without<Dead>);
type BotQueryFilterAliveNoTarget = (With<ClientEntity>, Without<Dead>, Without<BotCombatTarget>);

#[derive(Component)]
pub struct BotCombatTarget {
    entity: Entity,
}

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate)).add_systems(
            PreUpdate,
            (
                (
                    action_accept_party_invite,
                    action_attack_random_nearby_target,
                    action_attack_target,
                    action_attack_threat,
                    action_find_monster_spawn,
                    action_join_zone,
                    action_party_invite_nearby_bot,
                    action_pickup_nearest_item_drop,
                    action_revive_current_zone,
                    action_sit_recover_hp,
                    action_snowball_fight,
                    action_use_attack_skill,
                    action_use_buff_skill,
                )
                    .in_set(BigBrainSet::Actions),
                (
                    score_can_party_invite_nearby_bot,
                    score_find_nearby_item_drop_system,
                    score_find_nearby_target,
                    score_has_party_invite,
                    score_is_dead,
                    score_is_teleporting,
                    score_should_attack_target,
                    score_should_sit_recover_hp,
                    score_should_use_attack_skill,
                    score_should_use_buff_skill,
                    score_threat_is_not_target,
                )
                    .in_set(BigBrainSet::Scorers),
            ),
        );
    }
}

pub fn bot_thinker() -> ThinkerBuilder {
    Thinker::build()
        .picker(Highest)
        .when(IsDead { score: 1.0 }, ReviveCurrentZone)
        .when(IsTeleporting { score: 1.0 }, JoinZone)
        .when(HasPartyInvite { score: 1.0 }, AcceptPartyInvite)
        .when(ThreatIsNotTarget { score: 0.9 }, AttackThreat)
        .when(ShouldUseAttackSkill { score: 0.85 }, UseAttackSkill)
        .when(
            ShouldAttackTarget {
                min_score: 0.6,
                max_score: 0.8,
            },
            ActionAttackTarget,
        )
        .when(
            CanPartyInviteNearbyBot { score: 0.55 },
            PartyInviteNearbyBot,
        )
        .when(FindNearbyItemDrop { score: 0.5 }, PickupNearestItemDrop)
        .when(ShouldSitRecoverHp { score: 0.4 }, SitRecoverHp)
        .when(ShouldUseBuffSkill { score: 0.3 }, UseBuffSkill)
        .when(FindNearbyTarget { score: 0.2 }, AttackRandomNearbyTarget)
        .otherwise(FindMonsterSpawns)
}

pub fn bot_snowball_fight() -> ThinkerBuilder {
    Thinker::build()
        .picker(Highest)
        .otherwise(SnowballFight::default())
}
