use bevy::{
    ecs::query::WorldQuery,
    prelude::{Commands, Entity, EventReader, Query, Res, ResMut, Vec3, With},
};
use rand::Rng;

use rose_game_common::components::{AbilityValues, CharacterInfo, HealthPoints, ManaPoints};

use crate::game::{
    bundles::client_entity_teleport_zone,
    components::{
        ClientEntity, ClientEntitySector, Command, DamageSources, Dead, GameClient, MoveMode,
        NextCommand, PassiveRecoveryTime, Position, StatusEffects,
    },
    events::{ReviveEvent, RevivePosition},
    resources::ClientEntityList,
    GameData,
};

const REVIVE_SPAWN_RADIUS: f32 = 500.0;

#[derive(WorldQuery)]
pub struct ReviveEntityQuery<'w> {
    entity: Entity,

    ability_values: &'w AbilityValues,
    client_entity: &'w ClientEntity,
    client_entity_sector: &'w ClientEntitySector,
    character_info: &'w CharacterInfo,
    position: &'w Position,

    game_client: Option<&'w GameClient>,
}

pub fn revive_event_system(
    mut commands: Commands,
    mut events: EventReader<ReviveEvent>,
    query: Query<ReviveEntityQuery, With<Dead>>,
    game_data: Res<GameData>,
    mut client_entity_list: ResMut<ClientEntityList>,
) {
    let mut rng = rand::thread_rng();

    for event in events.iter() {
        let Ok(entity) = query.get(event.entity) else {
            continue;
        };

        let mut new_position = match event.position {
            RevivePosition::CurrentZone => {
                let revive_position =
                    if let Some(zone_data) = game_data.zones.get_zone(entity.position.zone_id) {
                        if let Some(revive_position) =
                            zone_data.get_closest_revive_position(entity.position.position)
                        {
                            revive_position
                        } else {
                            zone_data.start_position
                        }
                    } else {
                        entity.position.position
                    };

                Position::new(revive_position, entity.position.zone_id)
            }
            RevivePosition::SaveZone => Position::new(
                entity.character_info.revive_position,
                entity.character_info.revive_zone_id,
            ),
        };

        // Randomise respawn position
        new_position.position = Vec3::new(
            new_position.position.x + rng.gen_range(-REVIVE_SPAWN_RADIUS..=REVIVE_SPAWN_RADIUS),
            new_position.position.y + rng.gen_range(-REVIVE_SPAWN_RADIUS..=REVIVE_SPAWN_RADIUS),
            new_position.position.z,
        );

        // Reset entity state
        commands.entity(entity.entity).remove::<Dead>().insert((
            HealthPoints::new((3 * entity.ability_values.get_max_health()) / 10),
            ManaPoints::new((3 * entity.ability_values.get_max_mana()) / 10),
            StatusEffects::default(),
            MoveMode::Run,
            Command::with_stop(),
            NextCommand::default(),
            DamageSources::default(),
            PassiveRecoveryTime::default(),
        ));

        // Teleport to respawn position
        client_entity_teleport_zone(
            &mut commands,
            &mut client_entity_list,
            entity.entity,
            entity.client_entity,
            entity.client_entity_sector,
            entity.position,
            new_position.clone(),
            entity.game_client,
        );
    }
}
