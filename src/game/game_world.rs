use bevy::{
    ecs::{
        event::Events,
        prelude::{Schedule, StageLabel, World},
        schedule::{ShouldRun, SystemStage},
    },
    prelude::ParallelSystemDescriptorCoercion,
};
use chrono::Local;
use crossbeam_channel::Receiver;
use log::debug;
use std::time::{Duration, Instant};

use crate::game::{
    events::{
        ChatCommandEvent, DamageEvent, NpcStoreEvent, PartyEvent, PartyMemberEvent,
        PersonalStoreEvent, QuestTriggerEvent, RewardItemEvent, RewardXpEvent, SaveEvent,
        SkillEvent, UseItemEvent,
    },
    messages::control::ControlMessage,
    resources::{
        BotList, ClientEntityList, ControlChannel, GameConfig, GameData, LoginTokens, ServerList,
        ServerMessages, ServerTime, WorldRates, WorldTime, ZoneList,
    },
    systems::{
        ability_values_changed_system, ability_values_update_character_system,
        ability_values_update_npc_system, bot_ai_system, chat_commands_system,
        client_entity_visibility_system, command_system, control_server_system, damage_system,
        experience_points_system, expire_time_system, game_server_authentication_system,
        game_server_join_system, game_server_main_system, login_server_authentication_system,
        login_server_system, monster_spawn_system, npc_ai_system, npc_store_system,
        party_member_event_system, party_member_update_info_system, party_system,
        party_update_average_level_system, passive_recovery_system, personal_store_system,
        quest_system, reward_item_system, save_system, server_messages_system, skill_effect_system,
        startup_zones_system, status_effect_system, update_position_system, use_item_system,
        weight_system, world_server_authentication_system, world_server_system, world_time_system,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, StageLabel)]
enum GameStages {
    Startup,
    First,
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Output,
}

pub struct GameWorld {
    tick_rate_hz: u64,
    control_rx: Receiver<ControlMessage>,
}

impl GameWorld {
    pub fn new(control_rx: Receiver<ControlMessage>) -> Self {
        Self {
            tick_rate_hz: 30,
            control_rx,
        }
    }

    pub fn run(&mut self, game_config: GameConfig, game_data: GameData) {
        let mut world = World::new();
        world.insert_resource(BotList::new());
        world.insert_resource(ClientEntityList::new(&game_data.zones));
        world.insert_resource(ControlChannel::new(self.control_rx.clone()));
        world.insert_resource(LoginTokens::new());
        world.insert_resource(ServerList::new());
        world.insert_resource(ServerMessages::new());
        world.insert_resource(WorldRates::new());
        world.insert_resource(WorldTime::new());
        world.insert_resource(ZoneList::new());
        world.insert_resource(game_config);
        world.insert_resource(game_data);

        world.insert_resource(Events::<ChatCommandEvent>::default());
        world.insert_resource(Events::<DamageEvent>::default());
        world.insert_resource(Events::<NpcStoreEvent>::default());
        world.insert_resource(Events::<PartyEvent>::default());
        world.insert_resource(Events::<PartyMemberEvent>::default());
        world.insert_resource(Events::<PersonalStoreEvent>::default());
        world.insert_resource(Events::<QuestTriggerEvent>::default());
        world.insert_resource(Events::<RewardItemEvent>::default());
        world.insert_resource(Events::<RewardXpEvent>::default());
        world.insert_resource(Events::<SaveEvent>::default());
        world.insert_resource(Events::<SkillEvent>::default());
        world.insert_resource(Events::<UseItemEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_stage(
            GameStages::Startup,
            SystemStage::single_threaded()
                .with_run_criteria(ShouldRun::once)
                .with_system(startup_zones_system),
        );
        schedule.add_stage_after(
            GameStages::Startup,
            GameStages::First,
            SystemStage::parallel()
                .with_system(Events::<ChatCommandEvent>::update_system)
                .with_system(Events::<DamageEvent>::update_system)
                .with_system(Events::<NpcStoreEvent>::update_system)
                .with_system(Events::<PartyEvent>::update_system)
                .with_system(Events::<PartyMemberEvent>::update_system)
                .with_system(Events::<PersonalStoreEvent>::update_system)
                .with_system(Events::<QuestTriggerEvent>::update_system)
                .with_system(Events::<RewardItemEvent>::update_system)
                .with_system(Events::<RewardXpEvent>::update_system)
                .with_system(Events::<SaveEvent>::update_system)
                .with_system(Events::<SkillEvent>::update_system)
                .with_system(Events::<UseItemEvent>::update_system),
        );
        schedule.add_stage_after(
            GameStages::First,
            GameStages::Input,
            SystemStage::parallel()
                .with_system(world_time_system)
                .with_system(control_server_system)
                .with_system(login_server_authentication_system)
                .with_system(login_server_system)
                .with_system(world_server_authentication_system)
                .with_system(world_server_system)
                .with_system(game_server_authentication_system)
                .with_system(game_server_join_system)
                .with_system(game_server_main_system)
                .with_system(chat_commands_system)
                .with_system(monster_spawn_system)
                .with_system(bot_ai_system)
                .with_system(npc_ai_system)
                .with_system(expire_time_system)
                .with_system(status_effect_system)
                .with_system(passive_recovery_system),
        );

        schedule.add_stage_after(
            GameStages::Input,
            GameStages::PreUpdate,
            SystemStage::parallel()
                .with_system(command_system)
                .with_system(party_member_event_system)
                .with_system(party_system.after(party_member_event_system))
                .with_system(party_member_update_info_system.after(party_system))
                .with_system(update_position_system),
        );

        schedule.add_stage_after(
            GameStages::PreUpdate,
            GameStages::Update,
            SystemStage::parallel()
                .with_system(skill_effect_system)
                .with_system(personal_store_system)
                .with_system(npc_store_system)
                .with_system(damage_system)
                .with_system(quest_system)
                .with_system(use_item_system)
                .with_system(reward_item_system),
        );

        schedule.add_stage_after(
            GameStages::Update,
            GameStages::PostUpdate,
            SystemStage::parallel()
                .with_system(experience_points_system)
                .with_system(party_update_average_level_system.after(experience_points_system))
                .with_system(client_entity_visibility_system)
                .with_system(weight_system),
        );

        schedule.add_stage_after(
            GameStages::PostUpdate,
            GameStages::Output,
            SystemStage::parallel()
                .with_system(
                    ability_values_update_character_system.before(ability_values_changed_system),
                )
                .with_system(ability_values_update_npc_system.before(ability_values_changed_system))
                .with_system(ability_values_changed_system)
                .with_system(server_messages_system)
                .with_system(save_system),
        );

        let min_tick_duration = Duration::from_millis(1000 / self.tick_rate_hz);
        let mut last_tick = Instant::now();

        let mut tick_counter = 0;
        let mut tick_counter_duration = Duration::from_secs(0);
        let mut tick_counter_last_print = Instant::now();

        loop {
            let current_tick = Instant::now();
            world.insert_resource(ServerTime {
                delta: current_tick - last_tick,
                now: current_tick,
                local_time: Local::now(),
            });
            schedule.run_once(&mut world);

            let now = Instant::now();
            let tick_duration = now - current_tick;

            tick_counter += 1;
            tick_counter_duration += tick_duration;

            if now - tick_counter_last_print > Duration::from_secs(60) {
                let average_tick_duration =
                    tick_counter_duration.as_secs_f64() / (tick_counter as f64);
                debug!(
                    "Average tick duration: {:?}",
                    Duration::from_secs_f64(average_tick_duration)
                );

                tick_counter = 0;
                tick_counter_duration = Duration::from_secs(0);
                tick_counter_last_print = now;
            }

            if tick_duration < min_tick_duration {
                std::thread::sleep(min_tick_duration - tick_duration);
            }
            last_tick = current_tick;
        }
    }
}
