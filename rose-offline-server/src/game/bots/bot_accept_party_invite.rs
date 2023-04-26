use bevy::{
    ecs::query::WorldQuery,
    prelude::{Component, EventWriter, Query, With},
};
use big_brain::{
    prelude::{ActionBuilder, ActionState, ScorerBuilder},
    scorers::Score,
    thinker::Actor,
};

use crate::game::{components::PartyMembership, events::PartyEvent};

use super::BotQueryFilterAlive;

const PARTY_SEARCH_DISTANCE: f32 = 2000.0f32;

#[derive(Clone, Component, Debug, ScorerBuilder)]
pub struct HasPartyInvite {
    pub score: f32,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct AcceptPartyInvite;

#[derive(WorldQuery)]
pub struct BotQuery<'w> {
    party_membership: &'w PartyMembership,
}

pub fn score_has_party_invite(
    mut query: Query<(&HasPartyInvite, &Actor, &mut Score)>,
    query_bot: Query<BotQuery, BotQueryFilterAlive>,
) {
    for (scorer, &Actor(bot_entity), mut score) in query.iter_mut() {
        score.set(0.0);

        let Ok(bot) = query_bot.get(bot_entity) else {
            continue;
        };

        if bot.party_membership.party.is_some() {
            // We are already in a party
            continue;
        }

        if !bot.party_membership.pending_invites.is_empty() {
            score.set(scorer.score);
        }
    }
}

pub fn action_accept_party_invite(
    mut query: Query<(&Actor, &mut ActionState), With<AcceptPartyInvite>>,
    query_bot: Query<BotQuery, BotQueryFilterAlive>,
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
                if bot.party_membership.party.is_some() {
                    // Already in a party
                    *state = ActionState::Success;
                    continue;
                }

                if let Some(owner_entity) = bot.party_membership.pending_invites.get(0) {
                    party_events.send(PartyEvent::AcceptInvite {
                        owner_entity: *owner_entity,
                        invited_entity: bot_entity,
                    });

                    *state = ActionState::Success;
                } else {
                    *state = ActionState::Failure;
                }
            }
            ActionState::Cancelled => {
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
