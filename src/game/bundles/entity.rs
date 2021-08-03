use bevy_ecs::prelude::{Bundle, Commands, Entity, Mut};

use crate::{
    data::{AbilityValueCalculator, GetAbilityValues},
    game::{
        components::{
            AbilityValues, BasicStats, CharacterInfo, ClientEntity, ClientEntityId,
            ClientEntityType, ClientEntityVisibility, Command, Equipment, ExperiencePoints,
            GameClient, HealthPoints, Hotbar, Inventory, Level, ManaPoints, MotionData, MoveMode,
            MoveSpeed, NextCommand, Npc, NpcStandingDirection, Position, QuestState, SkillList,
            SkillPoints, SpawnOrigin, Stamina, StatPoints, StatusEffects, Team, UnionMembership,
        },
        messages::server::{ServerMessage, Teleport},
        resources::ClientEntityList,
    },
};

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

pub fn client_entity_recalculate_ability_values(
    commands: &mut Commands,
    ability_value_calculator: &dyn AbilityValueCalculator,
    client_entity: &ClientEntity,
    entity: Entity,
    status_effects: &StatusEffects,
    basic_stats: Option<&BasicStats>,
    character_info: Option<&CharacterInfo>,
    equipment: Option<&Equipment>,
    level: Option<&Level>,
    move_mode: Option<&MoveMode>,
    skill_list: Option<&SkillList>,
    npc: Option<&Npc>,
    health_points: Option<&mut Mut<HealthPoints>>,
    mana_points: Option<&mut Mut<ManaPoints>>,
) -> Option<AbilityValues> {
    // Update ability values
    let ability_values = if matches!(client_entity.entity_type, ClientEntityType::Character) {
        Some(ability_value_calculator.calculate(
            character_info.unwrap(),
            level.unwrap(),
            equipment.unwrap(),
            basic_stats.unwrap(),
            skill_list.unwrap(),
        ))
    } else if let Some(npc) = npc {
        ability_value_calculator.calculate_npc(npc.id)
    } else {
        None
    }?;

    if let Some(health_points) = health_points {
        let max_hp = (&ability_values, status_effects).get_max_health() as u32;
        if health_points.hp > max_hp {
            health_points.hp = max_hp;
        }
    }

    if let Some(mana_points) = mana_points {
        let max_mp = (&ability_values, status_effects).get_max_mana() as u32;
        if mana_points.mp > max_mp {
            mana_points.mp = max_mp;
        }
    }

    let mut entity_commands = commands.entity(entity);

    if let Some(move_mode) = move_mode {
        match move_mode {
            MoveMode::Run => {
                entity_commands.insert(MoveSpeed::new(
                    (&ability_values, status_effects).get_run_speed(),
                ));
            }
            MoveMode::Walk => {
                entity_commands.insert(MoveSpeed::new(
                    (&ability_values, status_effects).get_walk_speed(),
                ));
            }
        }
    }

    entity_commands.insert(ability_values.clone());
    Some(ability_values)
}
