use bevy_ecs::{
    event::EventWriter,
    prelude::{Commands, EventReader, Query, ResMut},
};
use log::{error, info};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{
        BasicStats, CharacterInfo, ClientEntity, ClientEntitySector, Equipment, ExperiencePoints,
        HealthPoints, Hotbar, Inventory, Level, ManaPoints, PartyMembership, Position, QuestState,
        SkillList, SkillPoints, Stamina, StatPoints, UnionMembership,
    },
    events::{PartyEvent, PartyMemberDisconnect, SaveEvent, SaveEventCharacter},
    resources::ClientEntityList,
    storage::character::CharacterStorage,
};

pub fn save_system(
    mut commands: Commands,
    query: Query<(
        Option<&ClientEntity>,
        Option<&ClientEntitySector>,
        &CharacterInfo,
        &BasicStats,
        &Inventory,
        &Equipment,
        &Level,
        &ExperiencePoints,
        &Position,
        &SkillList,
        &Hotbar,
        &HealthPoints,
        &ManaPoints,
        &SkillPoints,
        (
            &StatPoints,
            &QuestState,
            &UnionMembership,
            &Stamina,
            &PartyMembership,
        ),
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut save_events: EventReader<SaveEvent>,
    mut party_events: EventWriter<PartyEvent>,
) {
    for pending_save in save_events.iter() {
        match *pending_save {
            SaveEvent::Character(SaveEventCharacter {
                entity,
                remove_after_save,
            }) => {
                if let Ok((
                    client_entity,
                    client_entity_sector,
                    character_info,
                    basic_stats,
                    inventory,
                    equipment,
                    level,
                    experience_points,
                    position,
                    skill_list,
                    hotbar,
                    health_points,
                    mana_points,
                    skill_points,
                    (stat_points, quest_state, union_membership, stamina, party_membership),
                )) = query.get(entity)
                {
                    let storage = CharacterStorage {
                        info: character_info.clone(),
                        basic_stats: basic_stats.clone(),
                        inventory: inventory.clone(),
                        equipment: equipment.clone(),
                        level: level.clone(),
                        experience_points: experience_points.clone(),
                        position: position.clone(),
                        skill_list: skill_list.clone(),
                        hotbar: hotbar.clone(),
                        delete_time: None,
                        health_points: *health_points,
                        mana_points: *mana_points,
                        stat_points: *stat_points,
                        skill_points: *skill_points,
                        quest_state: quest_state.clone(),
                        union_membership: union_membership.clone(),
                        stamina: *stamina,
                    };

                    match storage.save() {
                        Ok(_) => info!("Saved character {}", character_info.name),
                        Err(error) => error!(
                            "Failed to save character {} with error {:?}",
                            character_info.name, error
                        ),
                    }

                    if remove_after_save {
                        if let (Some(client_entity), Some(client_entity_sector)) =
                            (client_entity, client_entity_sector)
                        {
                            client_entity_leave_zone(
                                &mut commands,
                                &mut client_entity_list,
                                entity,
                                client_entity,
                                client_entity_sector,
                                position,
                            );
                        }

                        if let PartyMembership::Member(party_entity) = party_membership {
                            party_events.send(PartyEvent::MemberDisconnect(
                                PartyMemberDisconnect {
                                    party_entity: *party_entity,
                                    disconnect_entity: entity,
                                    character_id: character_info.unique_id,
                                    name: character_info.name.clone(),
                                },
                            ));
                        }
                    }
                }

                if remove_after_save {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
