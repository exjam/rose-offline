use legion::{systems::CommandBuffer, Entity};

use crate::{
    data::AbilityValueCalculator,
    game::{
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityId,
            ClientEntityType, ClientEntityVisibility, Command, DamageSources, Equipment,
            ExperiencePoints, GameClient, HealthPoints, Hotbar, Inventory, Level, ManaPoints,
            MotionData, MoveMode, MoveSpeed, NextCommand, Npc, NpcAi, NpcStandingDirection,
            Position, QuestState, SkillList, SkillPoints, SpawnOrigin, Stamina, StatPoints,
            StatusEffects, Team, UnionMembership,
        },
        messages::server::{ServerMessage, Teleport},
        resources::ClientEntityList,
    },
};

pub fn create_character_entity(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    ability_values: AbilityValues,
    basic_stats: BasicStats,
    command: Command,
    equipment: Equipment,
    experience_points: ExperiencePoints,
    health_points: HealthPoints,
    hotbar: Hotbar,
    info: CharacterInfo,
    inventory: Inventory,
    level: Level,
    mana_points: ManaPoints,
    motion_data: MotionData,
    move_mode: MoveMode,
    move_speed: MoveSpeed,
    next_command: NextCommand,
    position: Position,
    quest_state: QuestState,
    skill_list: SkillList,
    skill_points: SkillPoints,
    stamina: Stamina,
    stat_points: StatPoints,
    team: Team,
    union_membership: UnionMembership,
) {
    cmd.add_component(*entity, ability_values);
    cmd.add_component(*entity, basic_stats);
    cmd.add_component(*entity, command);
    cmd.add_component(*entity, equipment);
    cmd.add_component(*entity, experience_points);
    cmd.add_component(*entity, health_points);
    cmd.add_component(*entity, hotbar);
    cmd.add_component(*entity, info);
    cmd.add_component(*entity, inventory);
    cmd.add_component(*entity, level);
    cmd.add_component(*entity, mana_points);
    cmd.add_component(*entity, motion_data);
    cmd.add_component(*entity, move_mode);
    cmd.add_component(*entity, move_speed);
    cmd.add_component(*entity, next_command);
    cmd.add_component(*entity, position);
    cmd.add_component(*entity, quest_state);
    cmd.add_component(*entity, skill_list);
    cmd.add_component(*entity, skill_points);
    cmd.add_component(*entity, stamina);
    cmd.add_component(*entity, stat_points);
    cmd.add_component(*entity, team);
    cmd.add_component(*entity, union_membership);
    cmd.add_component(*entity, StatusEffects::new());
}

pub fn create_npc_entity(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    ability_values: AbilityValues,
    command: Command,
    health_points: HealthPoints,
    level: Level,
    motion_data: MotionData,
    move_mode: MoveMode,
    move_speed: MoveSpeed,
    next_command: NextCommand,
    npc: Npc,
    npc_ai: Option<NpcAi>,
    position: Position,
    standing_direction: NpcStandingDirection,
    team: Team,
) {
    cmd.add_component(*entity, ability_values);
    cmd.add_component(*entity, command);
    cmd.add_component(*entity, health_points);
    cmd.add_component(*entity, level);
    cmd.add_component(*entity, motion_data);
    cmd.add_component(*entity, move_mode);
    cmd.add_component(*entity, move_speed);
    cmd.add_component(*entity, next_command);
    cmd.add_component(*entity, npc);
    if let Some(npc_ai) = npc_ai {
        cmd.add_component(*entity, npc_ai);
    }
    cmd.add_component(*entity, position);
    cmd.add_component(*entity, standing_direction);
    cmd.add_component(*entity, team);
    cmd.add_component(*entity, StatusEffects::new());
}

pub fn create_monster_entity(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    ability_values: AbilityValues,
    command: Command,
    damage_sources: Option<DamageSources>,
    health_points: HealthPoints,
    level: Level,
    motion_data: MotionData,
    move_mode: MoveMode,
    move_speed: MoveSpeed,
    next_command: NextCommand,
    npc: Npc,
    npc_ai: Option<NpcAi>,
    position: Position,
    spawn_origin: SpawnOrigin,
    team: Team,
) {
    cmd.add_component(*entity, ability_values);
    cmd.add_component(*entity, command);
    if let Some(damage_sources) = damage_sources {
        cmd.add_component(*entity, damage_sources);
    }
    cmd.add_component(*entity, health_points);
    cmd.add_component(*entity, level);
    cmd.add_component(*entity, motion_data);
    cmd.add_component(*entity, move_mode);
    cmd.add_component(*entity, move_speed);
    cmd.add_component(*entity, next_command);
    cmd.add_component(*entity, npc);
    if let Some(npc_ai) = npc_ai {
        cmd.add_component(*entity, npc_ai);
    }
    cmd.add_component(*entity, position);
    cmd.add_component(*entity, spawn_origin);
    cmd.add_component(*entity, team);
    cmd.add_component(*entity, StatusEffects::new());
}

#[derive(Copy, Clone, Debug)]
pub enum ClientEntityJoinZoneError {
    InvalidZone,
    OutOfEntityId,
}

pub fn client_entity_join_zone(
    cmd: &mut CommandBuffer,
    client_entity_list: &mut ClientEntityList,
    entity: &Entity,
    client_entity_type: ClientEntityType,
    position: &Position,
) -> Result<ClientEntityId, ClientEntityJoinZoneError> {
    let zone = client_entity_list
        .get_zone_mut(position.zone_id)
        .ok_or(ClientEntityJoinZoneError::InvalidZone)?;
    let client_entity = zone
        .join_zone(client_entity_type, *entity, position.position)
        .ok_or(ClientEntityJoinZoneError::OutOfEntityId)?;

    let client_entity_id = client_entity.id;
    cmd.add_component(*entity, client_entity);
    Ok(client_entity_id)
}

pub fn client_entity_leave_zone(
    cmd: &mut CommandBuffer,
    client_entity_list: &mut ClientEntityList,
    entity: &Entity,
    client_entity: &ClientEntity,
    position: &Position,
) {
    if let Some(client_entity_zone) = client_entity_list.get_zone_mut(position.zone_id) {
        client_entity_zone.leave_zone(entity, client_entity);
    }
    cmd.remove_component::<ClientEntity>(*entity);
    cmd.remove_component::<ClientEntityVisibility>(*entity);
}

pub fn client_entity_teleport_zone(
    cmd: &mut CommandBuffer,
    client_entity_list: &mut ClientEntityList,
    entity: &Entity,
    client_entity: &ClientEntity,
    previous_position: &Position,
    new_position: Position,
    game_client: Option<&GameClient>,
) {
    client_entity_leave_zone(
        cmd,
        client_entity_list,
        entity,
        client_entity,
        previous_position,
    );
    cmd.add_component(*entity, Command::with_stop());
    cmd.add_component(*entity, NextCommand::with_stop());
    cmd.add_component(*entity, new_position.clone());

    if let Some(game_client) = game_client {
        game_client
            .server_message_tx
            .send(ServerMessage::Teleport(Teleport {
                entity_id: client_entity.id,
                zone_id: new_position.zone_id,
                x: new_position.position.x,
                y: new_position.position.y,
                run_mode: 1,  // TODO: Run mode
                ride_mode: 0, // TODO: Ride mode
            }))
            .ok();
    }
}
