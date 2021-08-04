use bevy_ecs::prelude::{Bundle, Commands, Entity};

use crate::game::{
    components::{
        AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityId, ClientEntityType,
        ClientEntityVisibility, Command, Equipment, ExperiencePoints, GameClient, HealthPoints,
        Hotbar, Inventory, Level, ManaPoints, MotionData, MoveMode, MoveSpeed, NextCommand, Npc,
        NpcStandingDirection, ObjectVariables, Position, QuestState, SkillList, SkillPoints,
        SpawnOrigin, Stamina, StatPoints, StatusEffects, Team, UnionMembership,
    },
    messages::server::{ServerMessage, Teleport},
    resources::ClientEntityList,
};

pub const NPC_OBJECT_VARIABLES_COUNT: usize = 20;
pub const MONSTER_OBJECT_VARIABLES_COUNT: usize = 5;

#[derive(Bundle)]
pub struct NpcBundle {
    pub ability_values: AbilityValues,
    pub command: Command,
    pub health_points: HealthPoints,
    pub level: Level,
    pub motion_data: MotionData,
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub next_command: NextCommand,
    pub npc: Npc,
    //pub npc_ai: Option<NpcAi>,
    pub object_variables: ObjectVariables,
    pub position: Position,
    pub standing_direction: NpcStandingDirection,
    pub status_effects: StatusEffects,
    pub team: Team,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub ability_values: AbilityValues,
    pub basic_stats: BasicStats,
    pub command: Command,
    pub equipment: Equipment,
    pub experience_points: ExperiencePoints,
    pub health_points: HealthPoints,
    pub hotbar: Hotbar,
    pub info: CharacterInfo,
    pub inventory: Inventory,
    pub level: Level,
    pub mana_points: ManaPoints,
    pub motion_data: MotionData,
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub next_command: NextCommand,
    pub position: Position,
    pub quest_state: QuestState,
    pub skill_list: SkillList,
    pub skill_points: SkillPoints,
    pub stamina: Stamina,
    pub stat_points: StatPoints,
    pub status_effects: StatusEffects,
    pub team: Team,
    pub union_membership: UnionMembership,
}

#[derive(Bundle)]
pub struct MonsterBundle {
    pub ability_values: AbilityValues,
    pub command: Command,
    //pub damage_sources: Option<DamageSources>,
    pub health_points: HealthPoints,
    pub level: Level,
    pub motion_data: MotionData,
    pub move_mode: MoveMode,
    pub move_speed: MoveSpeed,
    pub next_command: NextCommand,
    pub npc: Npc,
    //pub npc_ai: Option<NpcAi>,
    pub object_variables: ObjectVariables,
    pub position: Position,
    pub spawn_origin: SpawnOrigin,
    pub status_effects: StatusEffects,
    pub team: Team,
}

#[derive(Copy, Clone, Debug)]
pub enum ClientEntityJoinZoneError {
    InvalidZone,
    OutOfEntityId,
}

pub fn client_entity_join_zone(
    commands: &mut Commands,
    client_entity_list: &mut ClientEntityList,
    entity: Entity,
    client_entity_type: ClientEntityType,
    position: &Position,
) -> Result<ClientEntityId, ClientEntityJoinZoneError> {
    let zone = client_entity_list
        .get_zone_mut(position.zone_id)
        .ok_or(ClientEntityJoinZoneError::InvalidZone)?;
    let client_entity = zone
        .join_zone(client_entity_type, entity, position.position)
        .ok_or(ClientEntityJoinZoneError::OutOfEntityId)?;

    let client_entity_id = client_entity.id;
    commands.entity(entity).insert(client_entity);
    Ok(client_entity_id)
}

pub fn client_entity_leave_zone(
    commands: &mut Commands,
    client_entity_list: &mut ClientEntityList,
    entity: Entity,
    client_entity: &ClientEntity,
    position: &Position,
) {
    if let Some(client_entity_zone) = client_entity_list.get_zone_mut(position.zone_id) {
        client_entity_zone.leave_zone(entity, client_entity);
    }
    commands
        .entity(entity)
        .remove::<ClientEntity>()
        .remove::<ClientEntityVisibility>();
}

pub fn client_entity_teleport_zone(
    commands: &mut Commands,
    client_entity_list: &mut ClientEntityList,
    entity: Entity,
    client_entity: &ClientEntity,
    previous_position: &Position,
    new_position: Position,
    game_client: Option<&GameClient>,
) {
    client_entity_leave_zone(
        commands,
        client_entity_list,
        entity,
        client_entity,
        previous_position,
    );
    commands
        .entity(entity)
        .insert(Command::with_stop())
        .insert(NextCommand::with_stop())
        .insert(new_position.clone());

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
