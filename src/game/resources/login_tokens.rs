use bevy::{ecs::prelude::Entity, prelude::Resource};

pub struct LoginToken {
    pub username: String,
    pub token: u32,
    pub selected_world_server: Entity,
    pub selected_game_server: Entity,
    pub selected_character: String,
    pub login_client: Option<Entity>,
    pub world_client: Option<Entity>,
    pub game_client: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct LoginTokens {
    pub tokens: Vec<LoginToken>,
}

impl LoginTokens {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn generate(
        &mut self,
        username: String,
        login_client: Entity,
        selected_world_server: Entity,
        selected_game_server: Entity,
    ) -> u32 {
        let mut token = 0u32;
        while token == 0 || self.tokens.iter().any(|x| x.token == token) {
            token = rand::random();
        }
        self.tokens.push(LoginToken {
            username,
            token,
            selected_world_server,
            selected_game_server,
            selected_character: String::default(),
            login_client: Some(login_client),
            world_client: None,
            game_client: None,
        });
        token
    }

    pub fn find_username_token(&self, username: &str) -> Option<&LoginToken> {
        self.tokens.iter().find(|token| token.username == username)
    }

    pub fn get_token_mut(&mut self, token_id: u32) -> Option<&mut LoginToken> {
        self.tokens.iter_mut().find(|token| token.token == token_id)
    }
}
