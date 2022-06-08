use pluto_network::{ prelude::*, topics::*, client::Client };

use pluto_network::protos::auth::{
      *,
      auth_coordinator_challenge::*,
      auth_coordinator_session_token::*
};
use pluto_network::protos::_status::{ ErrorResponse, error_response::ErrorType };

use crate::DATABASE;
use crate::rumqttc::QoS;

use crate::logic::auth::*;

pub struct AuthHandler;

#[async_trait::async_trait]
impl Handler for AuthHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::Auth).into()
    }

    // todo: how to handle client id?
    async fn handle(&self, message: Message, client: Client) {
        let init_msg: AuthNodeInit = match message.parse() {
            Some(m) => m,
            // ignore malformed messages for now
            None => { return; },
        };

        let node_pubkey_bytes = init_msg.pubkey;
        let db = DATABASE.get().unwrap();

        let mut challenge_msg_wrapper = AuthNodeInit::response();

        // return an error in case of invalid pubkey, otherwise find or create node
        let node = match add_or_find_node_pubkey(db, node_pubkey_bytes.clone()).await {
            None => {
                let mut error_msg = ErrorResponse::default();
                error_msg.error = ErrorType::PUBKEY_LENGTH_INVALID.into();
                challenge_msg_wrapper.challenge_status = Some(Challenge_status::Error(error_msg));
                client.send(
                    topic!(Node::Auth).topic("a".to_owned()),
                    challenge_msg_wrapper.clone(),
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));

                return;
            },
            Some(node) => node
        };

        let mut challenge_msg = Challenge::default();

        let challenge_bytes = generate_challenge_bytes();
        challenge_msg.challenge = challenge_bytes.clone();
        challenge_msg_wrapper.challenge_status = Some(Challenge_status::Challenge(challenge_msg));

        // send challenge to client
        let challenge_response_msg: AuthNodeChallengeResponse = match client.send_and_listen(
            topic!(Node::Auth).topic("a".to_owned()),
            challenge_msg_wrapper,
            QoS::AtMostOnce,
            false,
            std::time::Duration::from_secs(5)
        ).await {
            Ok(msg) => msg,
            Err(e) => { return; }
        };

        let mut token_msg_wrapper = AuthNodeChallengeResponse::response();

        // verify challenge, and if it's correct, send a session token to node
        match verify_challenge(challenge_bytes.to_vec(),
                            challenge_response_msg.response,
                            node_pubkey_bytes) {
            Some(_) => {
                let session_token = create_session(db, node.node_id).await;
                let mut token_msg = SessionToken::default();
                token_msg.session_token = session_token;
                token_msg_wrapper.session_token_status =
                    Some(Session_token_status::SessionToken(token_msg));
            },
            None => {
                let mut error_msg = ErrorResponse::default();
                error_msg.error = ErrorType::CHALLENGE_RESPONSE_INVALID.into();
                token_msg_wrapper.session_token_status =
                    Some(Session_token_status::Error(error_msg));
            }
        }

        client.send(
            topic!(Node::Auth).topic("a".to_owned()),
            token_msg_wrapper,
            QoS::AtMostOnce,
            false,
        ).await.err().map(|e| debug!("{e:?}"));
    }
}
