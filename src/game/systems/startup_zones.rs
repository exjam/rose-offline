use legion::{system, systems::CommandBuffer};

use crate::game::{
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
        let client_entity_zone = client_entity_list.get_zone_mut(zone_id).unwrap();

        // Create the Zone entity
        cmd.push((Zone { id: zone_id as u16 },));

        // Create the MonsterSpawnPoint entities
        for spawn in zone_data.monster_spawns.iter() {
            cmd.push((
                MonsterSpawnPoint::from(spawn),
                Position::new(spawn.position, zone_id as u16),
            ));
        }

        // Spawn all NPCs
        for npc in zone_data.npcs.iter() {
            let conversation_index = game_data
                .npcs
                .get_conversation(&npc.conversation)
                .map(|x| x.index)
                .unwrap_or(0);
            let entity = cmd.push((
                Npc::new(npc.npc.0 as u32, conversation_index as u16),
                NpcStandingDirection::new(npc.direction),
                Position::new(npc.position, zone_id as u16),
                Team::default_npc(),
                Command::default(),
                NextCommand::default(),
                game_data.npcs.get_npc_motions(npc.npc.0),
            ));

            if let Some(npc_data) = game_data.npcs.get_npc(npc.npc.0) {
                cmd.add_component(entity, HealthPoints::new(npc_data.health_points as u32));
                cmd.add_component(entity, Level::new(npc_data.level as u16));
                cmd.add_component(entity, MoveSpeed::new(npc_data.walk_speed as f32));
            }

            let ai_file_index = game_data
                .npcs
                .get_npc(npc.npc.0)
                .map(|npc_data| npc_data.ai_file_index)
                .unwrap_or(0);
            if ai_file_index != 0 {
                cmd.add_component(entity, NpcAi::new(ai_file_index as usize));
            }

            if let Some(ability_values) = game_data
                .ability_value_calculator
                .calculate_npc(npc.npc.0 as usize)
            {
                cmd.add_component(entity, ability_values);
            }

            cmd.add_component(
                entity,
                client_entity_zone
                    .allocate(ClientEntityType::Npc, entity, npc.position)
                    .unwrap(),
            );
        }
    }
}
