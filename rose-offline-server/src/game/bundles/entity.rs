use bevy::{
    ecs::prelude::{Bundle, Commands, Entity},
    math::Vec3,
    time::Time,
};
use rand::Rng;
use std::time::Duration;

use rose_data::{NpcId, ZoneId};

use crate::game::{
    components::{
        AbilityValues, Bank, BasicStats, CharacterInfo, ClanMembership, ClientEntity,
        ClientEntityId, ClientEntitySector, ClientEntityType, ClientEntityVisibility, Command,
        DamageSources, DroppedItem, EntityExpireTime, Equipment, ExperiencePoints, GameClient,
        HealthPoints, Hotbar, Inventory, ItemDrop, Level, ManaPoints, MotionData, MoveMode,
        MoveSpeed, NextCommand, Npc, NpcAi, NpcStandingDirection, ObjectVariables, Owner,
        OwnerExpireTime, PartyMembership, PartyOwner, PassiveRecoveryTime, Position, QuestState,
        SkillList, SkillPoints, SpawnOrigin, Stamina, StatPoints, StatusEffects,
        StatusEffectsRegen, Team, UnionMembership,
    },
    messages::server::{ServerMessage, Teleport},
    resources::ClientEntityList,
    GameData,
};

pub const EVENT_OBJECT_VARIABLES_COUNT: usize = 20;
pub const NPC_OBJECT_VARIABLES_COUNT: usize = 20;
pub const MONSTER_OBJECT_VARIABLES_COUNT: usize = 5;
pub const ITEM_DROP_ENTITY_EXPIRE_TIME: Duration = Duration::from_secs(120);
pub const ITEM_DROP_OWNER_EXPIRE_TIME: Duration = Duration::from_secs(60);
pub const ITEM_DROP_RADIUS: i32 = 200;

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
    pub status_effects_regen: StatusEffectsRegen,
    pub team: Team,
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub ability_values: AbilityValues,
    pub basic_stats: BasicStats,
    pub bank: Bank,
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
    pub party_membership: PartyMembership,
    pub passive_recovery_time: PassiveRecoveryTime,
    pub position: Position,
    pub quest_state: QuestState,
    pub skill_list: SkillList,
    pub skill_points: SkillPoints,
    pub stamina: Stamina,
    pub stat_points: StatPoints,
    pub status_effects: StatusEffects,
    pub status_effects_regen: StatusEffectsRegen,
    pub team: Team,
    pub union_membership: UnionMembership,
    pub clan_membership: ClanMembership,
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
    pub status_effects_regen: StatusEffectsRegen,
    pub team: Team,
}

impl MonsterBundle {
    pub fn spawn(
        commands: &mut Commands,
        client_entity_list: &mut ClientEntityList,
        game_data: &GameData,
        npc_id: NpcId,
        spawn_zone: ZoneId,
        spawn_origin: SpawnOrigin,
        spawn_range: i32,
        team: Team,
        owner: Option<(Entity, &Level)>,
        summon_skill_level: Option<i32>,
    ) -> Option<Entity> {
        let npc_data = game_data.npcs.get_npc(npc_id)?;
        let npc_ai = Some(npc_data.ai_file_index)
            .filter(|ai_file_index| *ai_file_index != 0)
            .map(|ai_file_index| NpcAi::new(ai_file_index as usize));

        let status_effects = StatusEffects::new();
        let status_effects_regen = StatusEffectsRegen::new();

        let ability_values = game_data.ability_value_calculator.calculate_npc(
            npc_id,
            &status_effects,
            owner.map(|(_, owner_level)| owner_level.level as i32),
            summon_skill_level,
        )?;

        let damage_sources = Some(ability_values.get_max_damage_sources())
            .filter(|max_damage_sources| *max_damage_sources > 0)
            .map(DamageSources::new);
        let health_points = HealthPoints::new(ability_values.get_max_health());
        let level = Level::new(ability_values.get_level() as u32);
        let move_mode = MoveMode::Walk;
        let move_speed = MoveSpeed::new(ability_values.get_walk_speed());

        let spawn_position = match spawn_origin {
            SpawnOrigin::Summoned(_, spawn_position) => spawn_position,
            SpawnOrigin::MonsterSpawnPoint(_, spawn_position) => spawn_position,
            SpawnOrigin::Quest(_, spawn_position) => spawn_position,
        };

        let position = Position::new(
            Vec3::new(
                spawn_position.x + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                spawn_position.y + rand::thread_rng().gen_range(-spawn_range..spawn_range) as f32,
                0.0,
            ),
            spawn_zone,
        );

        let mut entity_commands = commands.spawn(MonsterBundle {
            ability_values,
            command: Command::default(),
            health_points,
            level,
            motion_data: MotionData::from_npc(&game_data.npcs, npc_id),
            move_mode,
            move_speed,
            next_command: NextCommand::default(),
            npc: Npc::new(npc_id, 0),
            object_variables: ObjectVariables::new(MONSTER_OBJECT_VARIABLES_COUNT),
            position: position.clone(),
            status_effects,
            status_effects_regen,
            spawn_origin,
            team,
        });
        let entity = entity_commands.id();

        if let Some(damage_sources) = damage_sources {
            entity_commands.insert(damage_sources);
        }

        if let Some(npc_ai) = npc_ai {
            entity_commands.insert(npc_ai);
        }

        if let Some((owner_entity, ..)) = owner {
            entity_commands.insert(Owner::new(owner_entity));
        }

        client_entity_join_zone(
            commands,
            client_entity_list,
            entity,
            ClientEntityType::Monster,
            &position,
        )
        .expect("Failed to join monster into zone");

        Some(entity)
    }
}

#[derive(Bundle)]
pub struct ItemDropBundle {
    pub drop: ItemDrop,
    pub position: Position,
    pub entity_expire_time: EntityExpireTime,
    // pub owner: Option<Owner>
    // pub owner_expire_time: Option<OwnerExpireTime>
}

impl ItemDropBundle {
    pub fn spawn(
        commands: &mut Commands,
        client_entity_list: &mut ClientEntityList,
        item: DroppedItem,
        position: &Position,
        owner_entity: Option<Entity>,
        party_owner_entity: Option<Entity>,
        time: &Time,
    ) -> Option<Entity> {
        let mut rng = rand::thread_rng();

        let drop_point = Vec3::new(
            position.position.x + rng.gen_range(-ITEM_DROP_RADIUS..=ITEM_DROP_RADIUS) as f32,
            position.position.y + rng.gen_range(-ITEM_DROP_RADIUS..=ITEM_DROP_RADIUS) as f32,
            position.position.z,
        );

        let drop_position = Position::new(drop_point, position.zone_id);

        let mut entity_commands = commands.spawn(ItemDropBundle {
            drop: ItemDrop::with_dropped_item(item),
            position: drop_position.clone(),
            entity_expire_time: EntityExpireTime::new(
                time.last_update().unwrap() + ITEM_DROP_ENTITY_EXPIRE_TIME,
            ),
        });
        let entity = entity_commands.id();

        if let Some(owner_entity) = owner_entity {
            entity_commands.insert((
                Owner::new(owner_entity),
                OwnerExpireTime::new(time.last_update().unwrap() + ITEM_DROP_OWNER_EXPIRE_TIME),
            ));
        }

        if let Some(party_owner_entity) = party_owner_entity {
            entity_commands.insert(PartyOwner::new(party_owner_entity));
        }

        client_entity_join_zone(
            commands,
            client_entity_list,
            entity,
            ClientEntityType::ItemDrop,
            &drop_position,
        )
        .expect("Failed to drop item into zone");

        Some(entity)
    }
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
    let (client_entity, client_entity_sector) = zone
        .join_zone(client_entity_type, entity, position.position)
        .ok_or(ClientEntityJoinZoneError::OutOfEntityId)?;

    let client_entity_id = client_entity.id;
    commands
        .entity(entity)
        .insert((client_entity, client_entity_sector));
    Ok(client_entity_id)
}

pub fn client_entity_leave_zone(
    commands: &mut Commands,
    client_entity_list: &mut ClientEntityList,
    entity: Entity,
    client_entity: &ClientEntity,
    client_entity_sector: &ClientEntitySector,
    position: &Position,
) {
    if let Some(client_entity_zone) = client_entity_list.get_zone_mut(position.zone_id) {
        client_entity_zone.leave_zone(entity, client_entity, client_entity_sector);
    }
    commands
        .entity(entity)
        .remove::<(ClientEntity, ClientEntitySector, ClientEntityVisibility)>();
}

pub fn client_entity_teleport_zone(
    commands: &mut Commands,
    client_entity_list: &mut ClientEntityList,
    entity: Entity,
    client_entity: &ClientEntity,
    client_entity_sector: &ClientEntitySector,
    previous_position: &Position,
    new_position: Position,
    game_client: Option<&GameClient>,
) {
    client_entity_leave_zone(
        commands,
        client_entity_list,
        entity,
        client_entity,
        client_entity_sector,
        previous_position,
    );
    commands.entity(entity).insert((
        Command::with_stop(),
        NextCommand::default(),
        new_position.clone(),
    ));

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
