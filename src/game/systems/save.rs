use bevy_ecs::prelude::{Commands, EventReader, Query, ResMut};
use log::{error, info};

use crate::{
    data::character::CharacterStorage,
    game::{
        bundles::client_entity_leave_zone,
        components::{
            BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints, HealthPoints,
            Hotbar, Inventory, Level, ManaPoints, Position, QuestState, SkillList, SkillPoints,
            Stamina, StatPoints, UnionMembership,
        },
        events::{SaveEvent, SaveEventCharacter},
        resources::ClientEntityList,
    },
};

#[allow(clippy::type_complexity)]
pub fn save_system(
    mut commands: Commands,
    query: Query<(
        Option<&ClientEntity>,
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
        &StatPoints,
        (&QuestState, &UnionMembership, &Stamina),
    )>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut save_events: EventReader<SaveEvent>,
) {
    for pending_save in save_events.iter() {
        match *pending_save {
            SaveEvent::Character(SaveEventCharacter {
                entity,
                remove_after_save,
            }) => {
                let (client_entity, position) = if let Ok((
                    client_entity,
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
                    stat_points,
                    (quest_state, union_membership, stamina),
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
                            "Failed to save character {} with error: {:?}",
                            character_info.name, error
                        ),
                    }

                    (client_entity, Some(position))
                } else {
                    (None, None)
                };

                if remove_after_save {
                    if let (Some(client_entity), Some(position)) = (client_entity, position) {
                        client_entity_leave_zone(
                            &mut commands,
                            &mut client_entity_list,
                            entity,
                            client_entity,
                            &position,
                        );
                    }

                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
