use bevy_ecs::prelude::Entity;

pub struct LoginToken {
    pub username: String,
    pub token: u32,
    pub selected_world_server: Entity,
    pub selected_game_server: Entity,
    pub selected_character: String,
}

#[derive(Default)]
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
        });
        token
    }
}
