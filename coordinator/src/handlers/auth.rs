use pluto_network::{ prelude::*, topics::*, client::Client };

use pluto_network::protos::shared::{ ErrorResponse, error_response::ErrorType };
use pluto_network::protos::auth::{
      *,
      auth_coordinator_success::*
};

use crate::DATABASE;
use crate::rumqttc::QoS;

use crate::logic::auth::*;

pub struct AuthHandler;

#[async_trait::async_trait]
impl Handler for AuthHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::Auth).into()
    }

    async fn handle(&self, message: Message, client: Client) {
        let init_msg: AuthNodeInit = match message.parse() {
            Some(m) => m,
            // ignore malformed messages for now
            None => { return; },
        };

        let node_pubkey_bytes = init_msg.pubkey;
        let node_topic_id = base64::encode_config(node_pubkey_bytes.clone(),
                                                  base64::URL_SAFE_NO_PAD);

        let db = DATABASE.get().unwrap();

        let mut success_msg_wrapper = AuthNodeInit::response();

        // return an error in case of invalid pubkey, otherwise find or create node
        match add_node_pubkey(db, node_pubkey_bytes.clone()).await {
            Err(e) => {
                let mut error_msg = ErrorResponse::default();

                error_msg.error = match e {
                    AuthError::InvalidPubkey => ErrorType::PUBKEY_LENGTH_INVALID.into(),
                    AuthError::DatabaseError => ErrorType::SERVER_ERROR.into(),
                    AuthError::AlreadyRegistered => ErrorType::ALREADY_REGISTERED.into()
                };

                error_msg.error = ErrorType::PUBKEY_LENGTH_INVALID.into();
                success_msg_wrapper.auth_status = Some(Auth_status::Error(error_msg));
                client.send(
                    topic!(Node::Auth).topic(node_topic_id),
                    success_msg_wrapper,
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));

                return;
            },
            Ok(_) => {
                let mut success_msg = auth_coordinator_success::Success::default();
                success_msg.success = true;
                success_msg_wrapper.auth_status = Some(Auth_status::Success(success_msg));

                client.send(
                    topic!(Node::Auth).topic(node_topic_id),
                    success_msg_wrapper,
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));

                debug!("registered node with pubkey {node_pubkey_bytes:?}");
            }
        };
    }
}
