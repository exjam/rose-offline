use bevy::ecs::prelude::{Commands, Res, ResMut};
use log::warn;

use crate::game::{
    bundles::{
        client_entity_join_zone, NpcBundle, EVENT_OBJECT_VARIABLES_COUNT,
        NPC_OBJECT_VARIABLES_COUNT,
    },
    components::{
        ClientEntityType, Command, EventObject, HealthPoints, Level, MonsterSpawnPoint, MotionData,
        MoveMode, MoveSpeed, NextCommand, Npc, NpcAi, NpcStandingDirection, ObjectVariables,
        Position, StatusEffects, StatusEffectsRegen, Team,
    },
    resources::{ClientEntityList, GameData, ZoneList},
    GameConfig,
};

pub fn startup_zones_system(
    mut commands: Commands,
    mut client_entity_list: ResMut<ClientEntityList>,
    game_config: Res<GameConfig>,
    game_data: Res<GameData>,
    mut zone_list: ResMut<ZoneList>,
) {
    for zone_data in game_data.zones.iter() {
        // Add to zone list
        zone_list.add_zone(zone_data.id);

        // Create the Event Object entities
        for event_object in zone_data.event_objects.iter() {
            let entity = commands
                .spawn((
                    EventObject::new(event_object.event_id),
                    Position::new(event_object.position, zone_data.id),
                    ObjectVariables::new(EVENT_OBJECT_VARIABLES_COUNT),
                ))
                .id();

            zone_list.add_event_object(
                zone_data.id,
                event_object.event_id,
                event_object.map_chunk_x,
                event_object.map_chunk_y,
                entity,
            );
        }

        // Create all Monster Spawn Points
        if game_config.enable_monster_spawns {
            for spawn in zone_data.monster_spawns.iter() {
                // Verify basic_spawns
                for (npc, _) in &spawn.basic_spawns {
                    if game_data.npcs.get_npc(*npc).is_none() {
                        warn!(
                            "Invalid monster spawn {} in zone {}",
                            npc.get(),
                            zone_data.id.get()
                        );
                    }
                }

                // Verify tactic_spawns
                for (npc, _) in &spawn.tactic_spawns {
                    if game_data.npcs.get_npc(*npc).is_none() {
                        warn!(
                            "Invalid monster spawn {} in zone {}",
                            npc.get(),
                            zone_data.id.get()
                        );
                    }
                }

                commands.spawn((
                    MonsterSpawnPoint::from(spawn),
                    Position::new(spawn.position, zone_data.id),
                ));
            }
        }

        // Spawn all NPCs
        if game_config.enable_npc_spawns {
            for npc in zone_data.npcs.iter() {
                let npc_data = game_data.npcs.get_npc(npc.npc_id);
                let status_effects = StatusEffects::new();
                let status_effects_regen = StatusEffectsRegen::new();
                let ability_values = game_data.ability_value_calculator.calculate_npc(
                    npc.npc_id,
                    &status_effects,
                    None,
                    None,
                );

                if npc_data.is_none() || ability_values.is_none() {
                    warn!(
                        "Tried to spawn invalid npc id {} for zone {}",
                        npc.npc_id.get(),
                        zone_data.id.get()
                    );
                    continue;
                }
                let ability_values = ability_values.unwrap();
                let npc_data = npc_data.unwrap();

                let conversation_index = game_data
                    .npcs
                    .get_conversation(&npc.conversation)
                    .map(|x| x.index)
                    .unwrap_or(0);

                let npc_ai = Some(npc_data.ai_file_index)
                    .filter(|ai_file_index| *ai_file_index != 0)
                    .map(|ai_file_index| NpcAi::new(ai_file_index as usize));

                let position = Position::new(npc.position, zone_data.id);
                let move_speed = MoveSpeed::new(ability_values.get_walk_speed());
                let level = Level::new(ability_values.get_level() as u32);
                let health_points = HealthPoints::new(ability_values.get_max_health());

                let mut entity_commands = commands.spawn(NpcBundle {
                    ability_values,
                    command: Command::default(),
                    health_points,
                    level,
                    motion_data: MotionData::from_npc(&game_data.npcs, npc.npc_id),
                    move_mode: MoveMode::Walk,
                    move_speed,
                    next_command: NextCommand::default(),
                    npc: Npc::new(npc.npc_id, conversation_index as u16),
                    object_variables: ObjectVariables::new(NPC_OBJECT_VARIABLES_COUNT),
                    position: position.clone(),
                    standing_direction: NpcStandingDirection::new(npc.direction),
                    status_effects,
                    status_effects_regen,
                    team: Team::default_npc(),
                });
                let entity = entity_commands.id();

                if let Some(npc_ai) = npc_ai {
                    entity_commands.insert(npc_ai);
                }

                client_entity_join_zone(
                    &mut commands,
                    &mut client_entity_list,
                    entity,
                    ClientEntityType::Npc,
                    &position,
                )
                .expect("Failed to join zone with NPC");

                zone_list.add_npc(npc.npc_id, entity);
            }
        }
    }
}
