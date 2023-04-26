use bevy::{
    ecs::query::WorldQuery,
    math::Vec3Swizzles,
    prelude::{Component, Entity, EventWriter, Query, Res, With},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use crate::game::{
    components::{ClientEntityType, Party, PartyMembership, Position},
    events::PartyEvent,
    resources::ClientEntityList,
};

use super::{create_bot::BotBuild, BotQueryFilterAlive};

const PARTY_SEARCH_DISTANCE: f32 = 2000.0f32;

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct CanPartyInviteNearbyBot {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct PartyInviteNearbyBot;

#[derive(WorldQuery)]
pub struct BotQuery<'w> {
    entity: Entity,
    bot_build: &'w BotBuild,
    party_membership: &'w PartyMembership,
    position: &'w Position,
}

pub fn score_can_party_invite_nearby_bot(
    mut query: Query<(&CanPartyInviteNearbyBot, &Actor, &mut Score)>,
    query_bot: Query<BotQuery, BotQueryFilterAlive>,
    query_party: Query<&Party>,
    client_entity_list: Res<ClientEntityList>,
) {
    for (scorer, &Actor(bot_entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok(bot) = query_bot.get(bot_entity) else {
            continue;
        };

        if let Some(party_entity) = bot.party_membership.party {
            let Ok(party) = query_party.get(party_entity) else {
                continue;
            };

            if party.owner != bot_entity {
                // Only party owner can send invites
                continue;
            }

            if party.members.is_full() {
                // Party is full
                continue;
            }
        }

        let Some(zone_entities) =
            client_entity_list.get_zone(bot.position.zone_id) else {
                continue;
            };

        // Are there any nearby bots which do not have a party
        if zone_entities
            .iter_entity_type_within_distance(
                bot.position.position.xy(),
                PARTY_SEARCH_DISTANCE,
                &[ClientEntityType::Character],
            )
            .any(|(nearby_entity, _)| {
                query_bot
                    .get(nearby_entity)
                    .ok()
                    .map_or(false, |nearby_bot| {
                        nearby_bot.party_membership.party.is_none()
                            && nearby_bot.bot_build.job_id != bot.bot_build.job_id
                    })
            })
        {
            score.set(scorer.score);
        }
    }
}

pub fn action_party_invite_nearby_bot(
    mut query: Query<(&Actor, &mut ActionState), With<PartyInviteNearbyBot>>,
    query_bot: Query<BotQuery, BotQueryFilterAlive>,
    client_entity_list: Res<ClientEntityList>,
    mut party_events: EventWriter<PartyEvent>,
) {
    for (&Actor(bot_entity), mut state) in query.iter_mut() {
        let Ok(bot) = query_bot.get(bot_entity) else {
            continue;
        };

        match *state {
            ActionState::Requested => {
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                let Some(zone_entities) =
                    client_entity_list.get_zone(bot.position.zone_id) else {
                        *state = ActionState::Failure;
                        continue;
                    };

                *state = ActionState::Failure;

                for nearby_bot in zone_entities
                    .iter_entity_type_within_distance(
                        bot.position.position.xy(),
                        PARTY_SEARCH_DISTANCE,
                        &[ClientEntityType::Character],
                    )
                    .filter_map(|(nearby_entity, _)| query_bot.get(nearby_entity).ok())
                {
                    if nearby_bot.party_membership.party.is_none()
                        && nearby_bot.bot_build.job_id != bot.bot_build.job_id
                    {
                        party_events.send(PartyEvent::Invite {
                            owner_entity: bot.entity,
                            invited_entity: nearby_bot.entity,
                        });
                        *state = ActionState::Success;
                        break;
                    }
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
