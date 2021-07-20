use legion::{system, systems::CommandBuffer};
use log::warn;

use crate::game::{
    bundles::{client_entity_join_zone, create_npc_entity},
    components::{
        ClientEntityType, Command, HealthPoints, Level, MonsterSpawnPoint, MoveSpeed, NextCommand,
        Npc, NpcAi, NpcStandingDirection, Position, Team, Zone,
    },
    resources::{ClientEntityList, GameData},
};

#[system]
pub fn startup_zones(
    cmd: &mut CommandBuffer,
    #[resource] client_entity_list: &mut ClientEntityList,
    #[resource] game_data: &GameData,
) {
    for (zone_id, zone_data) in game_data.zones.iter() {
        let zone_id = *zone_id as usize;

        // Create the Zone entity
        cmd.push((Zone { id: zone_id as u16 },));

        // Create the MonsterSpawnPoint entities
        for spawn in zone_data.monster_spawns.iter() {
            // Verify basic_spawns
            for (npc, _) in &spawn.basic_spawns {
                if game_data.npcs.get_npc(npc.0).is_none() {
                    warn!("Invalid monster spawn {} in zone {}", npc.0, zone_id);
                }
            }

            // Verify tactic_spawns
            for (npc, _) in &spawn.tactic_spawns {
                if game_data.npcs.get_npc(npc.0).is_none() {
                    warn!("Invalid monster spawn {} in zone {}", npc.0, zone_id);
                }
            }

            cmd.push((
                MonsterSpawnPoint::from(spawn),
                Position::new(spawn.position, zone_id as u16),
            ));
        }

        // Spawn all NPCs
        for npc in zone_data.npcs.iter() {
            let npc_data = game_data.npcs.get_npc(npc.npc.0);
            let ability_values = game_data
                .ability_value_calculator
                .calculate_npc(npc.npc.0 as usize);

            if npc_data.is_none() || ability_values.is_none() {
                warn!(
                    "Tried to spawn invalid npc id {} for zone {}",
                    npc.npc.0, zone_id
                );
                continue;
            }

            let conversation_index = game_data
                .npcs
                .get_conversation(&npc.conversation)
                .map(|x| x.index)
                .unwrap_or(0);

            let npc_data = npc_data.unwrap();
            let npc_ai = Some(npc_data.ai_file_index)
                .filter(|ai_file_index| *ai_file_index != 0)
                .map(|ai_file_index| NpcAi::new(ai_file_index as usize));

            let position = Position::new(npc.position, zone_id as u16);

            let ability_values = ability_values.unwrap();
            let health_points = HealthPoints::new(ability_values.max_health as u32);
            let level = Level::new(ability_values.level as u32);
            let move_speed = MoveSpeed::new(ability_values.walk_speed as f32);

            let entity = cmd.push(());

            create_npc_entity(
                cmd,
                &entity,
                ability_values,
                Command::default(),
                health_points,
                level,
                game_data.npcs.get_npc_motions(npc.npc.0),
                move_speed,
                NextCommand::default(),
                Npc::new(npc.npc.0 as u32, conversation_index as u16),
                npc_ai,
                position.clone(),
                NpcStandingDirection::new(npc.direction),
                Team::default_npc(),
            );

            client_entity_join_zone(
                cmd,
                client_entity_list,
                &entity,
                ClientEntityType::Npc,
                &position,
            )
            .expect("Failed to join zone with NPC");
        }
    }
}
