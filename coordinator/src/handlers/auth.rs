use pluto_network::{ prelude::*, topics::*, client::Client, utils::* };

use pluto_network::protos::shared::{ ErrorResponse, error_response::ErrorType };
use pluto_network::protos::auth::{
      *,
      auth_coordinator_success::*
};
use x25519_dalek::PublicKey;

use crate::{ DATABASE, COORDINATOR_PRIVKEY, COORDINATOR_PUBKEY };
use crate::rumqttc::QoS;

use crate::logic::auth::*;
use crate::coordinator::Coordinator;

pub struct AuthHandler;

#[async_trait::async_trait]
impl Handler for AuthHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::Auth).into()
    }

    async fn handle(&self, message: Message, client: Client) -> Option<()> {
        let init_msg: AuthNodeInit = match message.encrypted() {
            Ok(m) => {
                match m.decrypt(&COORDINATOR_PRIVKEY) {
                    Some(m) => m,
                    // todo return a error message
                    None => {
                        warn!("failed to decrypt");
                        return None;
                    }
                }
            },
            // ignore malformed messages for now
            Err(_) => {
                warn!("not encrypted");
                return None;
            },
        };

        let node_pubkey_bytes = init_msg.pubkey;
        let node_topic_id = get_node_topic_id(node_pubkey_bytes.clone());

        let db = DATABASE.get().unwrap();

        let mut success_msg_wrapper = AuthNodeInit::response();

        // return an error in case of invalid pubkey, otherwise find or create node
        match Coordinator::add_node_pubkey(db, node_pubkey_bytes.clone()).await {
            Err(e) => {
                let mut error_msg = ErrorResponse::default();

                error_msg.error = match e {
                    AuthError::InvalidPubkey => {
                        // ErrorType::PUBKEY_LENGTH_INVALID.into()
                        // pubkey is invalid -- we cannot send an encrypted reply. so just do nothing
                        return None
                    },
                    AuthError::DatabaseError => ErrorType::SERVER_ERROR.into(),
                    AuthError::AlreadyRegistered => ErrorType::ALREADY_REGISTERED.into()
                };

                success_msg_wrapper.auth_status = Some(Auth_status::Error(error_msg));

                // we've already checked that pubkey is the correct length
                let node_pubkey_arr: [u8; 32] = node_pubkey_bytes.try_into().unwrap();
                let node_pubkey = PublicKey::from(node_pubkey_arr);

                let success_msg_wrapper_enc =
                    success_msg_wrapper.encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

                client.send(
                    topic!(Node::Auth).topic(node_topic_id),
                    success_msg_wrapper_enc,
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));

                return None;
            },
            Ok(_) => {
                let mut success_msg = auth_coordinator_success::Success::default();
                success_msg.success = true;
                success_msg_wrapper.auth_status = Some(Auth_status::Success(success_msg));

                // we've already checked that pubkey is the correct length
                let node_pubkey_arr: [u8; 32] = node_pubkey_bytes.try_into().unwrap();
                let node_pubkey = PublicKey::from(node_pubkey_arr);

                let success_msg_wrapper_enc =
                    success_msg_wrapper.encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

                client.send(
                    topic!(Node::Auth).topic(node_topic_id),
                    success_msg_wrapper_enc,
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));

                debug!("Registered node with pubkey {node_pubkey_arr:?}");
            }
        };

        Some(())
    }
}
