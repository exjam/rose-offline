use bevy::ecs::{
    event::EventWriter,
    prelude::{Commands, EventReader, Query, ResMut},
    query::WorldQuery,
};
use log::{error, info};

use crate::game::{
    bundles::client_entity_leave_zone,
    components::{
        Account, Bank, BasicStats, CharacterInfo, ClanMembership, ClientEntity, ClientEntitySector,
        Equipment, ExperiencePoints, HealthPoints, Hotbar, Inventory, Level, ManaPoints,
        PartyMembership, Position, QuestState, SkillList, SkillPoints, Stamina, StatPoints,
        UnionMembership,
    },
    events::{ClanEvent, PartyMemberEvent, SaveEvent},
    resources::ClientEntityList,
    storage::{bank::BankStorage, character::CharacterStorage},
};

#[derive(WorldQuery)]
pub struct SaveEntityQuery<'w> {
    client_entity: Option<&'w ClientEntity>,
    client_entity_sector: Option<&'w ClientEntitySector>,
    account: &'w Account,
    character_info: &'w CharacterInfo,
    basic_stats: &'w BasicStats,
    bank: &'w Bank,
    inventory: &'w Inventory,
    equipment: &'w Equipment,
    level: &'w Level,
    experience_points: &'w ExperiencePoints,
    position: &'w Position,
    skill_list: &'w SkillList,
    hotbar: &'w Hotbar,
    health_points: &'w HealthPoints,
    mana_points: &'w ManaPoints,
    skill_points: &'w SkillPoints,
    stat_points: &'w StatPoints,
    quest_state: &'w QuestState,
    union_membership: &'w UnionMembership,
    stamina: &'w Stamina,
    party_membership: &'w PartyMembership,
    clan_membership: &'w ClanMembership,
}

pub fn save_system(
    mut commands: Commands,
    query: Query<SaveEntityQuery>,
    mut client_entity_list: ResMut<ClientEntityList>,
    mut save_events: EventReader<SaveEvent>,
    mut clan_events: EventWriter<ClanEvent>,
    mut party_member_events: EventWriter<PartyMemberEvent>,
) {
    for pending_save in save_events.iter() {
        match *pending_save {
            SaveEvent::Character {
                entity,
                remove_after_save,
            } => {
                if let Ok(character) = query.get(entity) {
                    let storage = CharacterStorage {
                        info: character.character_info.clone(),
                        basic_stats: character.basic_stats.clone(),
                        inventory: character.inventory.clone(),
                        equipment: character.equipment.clone(),
                        level: *character.level,
                        experience_points: character.experience_points.clone(),
                        position: character.position.clone(),
                        skill_list: character.skill_list.clone(),
                        hotbar: character.hotbar.clone(),
                        delete_time: None,
                        health_points: *character.health_points,
                        mana_points: *character.mana_points,
                        stat_points: *character.stat_points,
                        skill_points: *character.skill_points,
                        quest_state: character.quest_state.clone(),
                        union_membership: character.union_membership.clone(),
                        stamina: *character.stamina,
                    };
                    match storage.save() {
                        Ok(_) => info!("Saved character {}", &character.character_info.name),
                        Err(error) => error!(
                            "Failed to save character {} with error {:?}",
                            &character.character_info.name, error
                        ),
                    }

                    let bank_storage = BankStorage::from(character.bank);
                    match bank_storage.save(&character.account.name) {
                        Ok(_) => info!("Saved bank for account {}", &character.account.name),
                        Err(error) => error!(
                            "Failed to save bank for account {} with error {:?}",
                            &character.account.name, error
                        ),
                    }

                    if remove_after_save {
                        if let (Some(client_entity), Some(client_entity_sector)) =
                            (character.client_entity, character.client_entity_sector)
                        {
                            client_entity_leave_zone(
                                &mut commands,
                                &mut client_entity_list,
                                entity,
                                client_entity,
                                client_entity_sector,
                                character.position,
                            );
                        }

                        if let Some(party_entity) = character.party_membership.party() {
                            party_member_events.send(PartyMemberEvent::Disconnect {
                                party_entity,
                                disconnect_entity: entity,
                                character_id: character.character_info.unique_id,
                                name: character.character_info.name.clone(),
                            });
                        }

                        if let Some(&clan_entity) = character.clan_membership.as_ref() {
                            clan_events.send(ClanEvent::MemberDisconnect {
                                clan_entity,
                                disconnect_entity: entity,
                                name: character.character_info.name.clone(),
                                level: *character.level,
                                job: character.character_info.job,
                            });
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
