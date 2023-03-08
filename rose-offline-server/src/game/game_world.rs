use bevy::{
    app::ScheduleRunnerSettings,
    prelude::{
        App, CoreSet, IntoSystemAppConfigs, IntoSystemConfig, IntoSystemConfigs,
        IntoSystemSetConfig, SystemSet,
    }, MinimalPlugins,
};
use crossbeam_channel::Receiver;
use std::time::Duration;

use crate::game::{
    events::{
        BankEvent, ChatCommandEvent, ClanEvent, DamageEvent, ItemLifeEvent, NpcStoreEvent,
        PartyEvent, PartyMemberEvent, PersonalStoreEvent, PickupItemEvent, QuestTriggerEvent,
        RewardItemEvent, RewardXpEvent, SaveEvent, SkillEvent, UseItemEvent,
    },
    messages::control::ControlMessage,
    resources::{
        BotList, ClientEntityList, ControlChannel, GameConfig, GameData, LoginTokens, ServerList,
        ServerMessages, WorldRates, WorldTime, ZoneList,
    },
    systems::{
        ability_values_changed_system, ability_values_update_character_system,
        ability_values_update_npc_system, bank_system, bot_ai_system, chat_commands_system,
        clan_system, client_entity_visibility_system, command_system, control_server_system,
        damage_system, driving_time_system, experience_points_system, expire_time_system,
        game_server_authentication_system, game_server_join_system, game_server_main_system,
        item_life_system, login_server_authentication_system, login_server_system,
        monster_spawn_system, npc_ai_system, npc_store_system, party_member_event_system,
        party_member_update_info_system, party_system, party_update_average_level_system,
        passive_recovery_system, personal_store_system, pickup_item_system, quest_system,
        reward_item_system, save_system, server_messages_system, skill_effect_system,
        startup_clans_system, startup_zones_system, status_effect_system,
        update_character_motion_data_system, update_npc_motion_data_system, update_position_system,
        use_item_system, weight_system, world_server_authentication_system, world_server_system,
        world_time_system,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
#[system_set(base)]
enum GameStages {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Output,
}

pub struct GameWorld {
    control_rx: Receiver<ControlMessage>,
}

impl GameWorld {
    pub fn new(control_rx: Receiver<ControlMessage>) -> Self {
        Self { control_rx }
    }

    pub fn run(&mut self, game_config: GameConfig, game_data: GameData) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.insert_resource(BotList::new());
        app.insert_resource(ClientEntityList::new(&game_data.zones));
        app.insert_resource(ControlChannel::new(self.control_rx.clone()));
        app.insert_resource(LoginTokens::new());
        app.insert_resource(ServerList::new());
        app.insert_resource(ServerMessages::new());
        app.insert_resource(WorldRates::new());
        app.insert_resource(WorldTime::new());
        app.insert_resource(ZoneList::new());
        app.insert_resource(game_config);
        app.insert_resource(game_data);

        app.add_event::<BankEvent>();
        app.add_event::<ChatCommandEvent>();
        app.add_event::<ClanEvent>();
        app.add_event::<DamageEvent>();
        app.add_event::<ItemLifeEvent>();
        app.add_event::<NpcStoreEvent>();
        app.add_event::<PartyEvent>();
        app.add_event::<PartyMemberEvent>();
        app.add_event::<PersonalStoreEvent>();
        app.add_event::<PickupItemEvent>();
        app.add_event::<QuestTriggerEvent>();
        app.add_event::<RewardItemEvent>();
        app.add_event::<RewardXpEvent>();
        app.add_event::<SaveEvent>();
        app.add_event::<SkillEvent>();
        app.add_event::<UseItemEvent>();

        app.add_systems((startup_clans_system, startup_zones_system).on_startup());

        app.add_systems(
            (
                world_time_system,
                control_server_system,
                login_server_authentication_system,
                login_server_system,
                world_server_authentication_system,
                world_server_system,
                game_server_authentication_system,
                game_server_join_system,
                game_server_main_system,
                chat_commands_system,
                monster_spawn_system,
                bot_ai_system,
                npc_ai_system,
                expire_time_system,
                status_effect_system,
            )
                .in_base_set(GameStages::Input),
        );

        app.add_systems(
            (passive_recovery_system, driving_time_system).in_base_set(GameStages::Input),
        );

        app.add_systems(
            (
                update_character_motion_data_system.before(command_system),
                update_npc_motion_data_system.before(command_system),
                command_system,
                pickup_item_system.after(command_system),
                party_member_event_system,
                party_system.after(party_member_event_system),
                party_member_update_info_system.after(party_system),
                update_position_system,
                clan_system,
            )
                .in_base_set(GameStages::PreUpdate),
        );

        app.add_systems(
            (
                bank_system,
                skill_effect_system,
                personal_store_system,
                npc_store_system,
                damage_system,
                quest_system,
                use_item_system,
                reward_item_system,
            )
                .in_base_set(GameStages::Update),
        );

        app.add_systems(
            (
                item_life_system,
                experience_points_system,
                party_update_average_level_system.after(experience_points_system),
                client_entity_visibility_system,
                weight_system,
            )
                .in_base_set(GameStages::PostUpdate),
        );

        app.add_systems(
            (
                ability_values_update_character_system.before(ability_values_changed_system),
                ability_values_update_npc_system.before(ability_values_changed_system),
                ability_values_changed_system,
                server_messages_system,
                save_system,
            )
                .in_base_set(GameStages::Output),
        );

        app.configure_set(GameStages::Input.after(CoreSet::Update));
        app.configure_set(GameStages::PreUpdate.after(GameStages::Input));
        app.configure_set(GameStages::Update.after(GameStages::PreUpdate));
        app.configure_set(GameStages::PostUpdate.after(GameStages::Update));
        app.configure_set(GameStages::Output.after(GameStages::PostUpdate));

        app.insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 30.0,
        )));

        app.run();
    }
}
