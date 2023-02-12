use bevy::prelude::Commands;

use rose_data::QuestTriggerHash;
use rose_game_common::components::ClanUniqueId;

use crate::game::{
    components::{Clan, ClanMember, Level},
    storage::{character::CharacterStorage, clan::ClanStorage},
};

pub fn startup_clans_system(mut commands: Commands) {
    let clans = ClanStorage::try_load_clan_list().unwrap_or_default();
    for clan_storage in clans {
        let mut members = Vec::new();

        for member in clan_storage.members {
            if let Ok(character) = CharacterStorage::try_load(&member.name) {
                members.push(ClanMember::Offline {
                    name: member.name,
                    position: member.position,
                    contribution: member.contribution,
                    level: Level::new(character.level.level),
                    job: character.info.job,
                });
            }
        }

        commands.spawn(Clan {
            unique_id: ClanUniqueId::new(QuestTriggerHash::from(clan_storage.name.as_str()).hash)
                .unwrap(),
            name: clan_storage.name,
            description: clan_storage.description,
            mark: clan_storage.mark,
            money: clan_storage.money,
            points: clan_storage.points,
            level: clan_storage.level,
            skills: clan_storage.skills,
            members,
        });
    }
}
