use legion::{system, systems::CommandBuffer, world::SubWorld, Query};

use crate::{
    data::character::CharacterStorage,
    game::{
        bundles::client_entity_leave_zone,
        components::{
            BasicStats, CharacterInfo, ClientEntity, Equipment, ExperiencePoints, HealthPoints,
            Hotbar, Inventory, Level, ManaPoints, Position, QuestState, SkillList, SkillPoints,
            Stamina, StatPoints, UnionMembership,
        },
        resources::{ClientEntityList, PendingCharacterSave, PendingSave, PendingSaveList},
    },
};

#[allow(clippy::type_complexity)]
#[system]
pub fn save(
    cmd: &mut CommandBuffer,
    world: &mut SubWorld,
    entity_query: &mut Query<(
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
        &QuestState,
        &UnionMembership,
        &Stamina,
    )>,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] pending_save_list: &mut PendingSaveList,
) {
    for pending_save in pending_save_list.iter() {
        match *pending_save {
            PendingSave::Character(PendingCharacterSave {
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
                    quest_state,
                    union_membership,
                    stamina,
                )) = entity_query.get(world, entity)
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
                    storage.save().ok();

                    (client_entity, Some(position))
                } else {
                    (None, None)
                };

                if remove_after_save {
                    if let (Some(client_entity), Some(position)) = (client_entity, position) {
                        client_entity_leave_zone(
                            cmd,
                            client_entity_list,
                            &entity,
                            client_entity,
                            &position,
                        );
                    }

                    cmd.remove(entity);
                }
            }
        }
    }

    pending_save_list.clear();
}
