use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

use crate::game::messages::client::{
    ClientMessage, ConnectionRequestError, ConnectionRequestResponse, GetChannelListError,
    JoinServerError, JoinServerResponse, LoginError,
};
use crate::game::{
    components::{Account, AccountError, WorldClient},
    resources::LoginTokens,
    resources::ServerList,
};

#[system(for_each)]
pub fn world_server_authentication(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    client: &mut WorldClient,
    #[resource] login_tokens: &mut LoginTokens,
) {
    if let Ok(message) = client.client_message_rx.try_recv() {
        match message {
            ClientMessage::ConnectionRequest(message) => {
                let response = if let Some((login_token, password_md5)) = message.login_token {
                    if let Some(username) = login_tokens
                        .tokens
                        .iter()
                        .find(|t| t.token == login_token)
                        .and_then(|t| Some(&t.username))
                    {
                        match Account::try_load(username, &password_md5) {
                            Ok(account) => {
                                cmd.add_component(*entity, account);
                                Ok(ConnectionRequestResponse {
                                    packet_sequence_id: 123,
                                })
                            }
                            Err(error) => Err(match error {
                                AccountError::InvalidPassword => {
                                    ConnectionRequestError::InvalidPassword
                                }
                                _ => ConnectionRequestError::Failed,
                            }),
                        }
                    } else {
                        Err(ConnectionRequestError::InvalidId)
                    }
                } else {
                    Err(ConnectionRequestError::Failed)
                };

                message.response_tx.send(response).ok();
            }
            _ => {
                client.pending_messages.push_back(message);
            }
        }
    }
}
