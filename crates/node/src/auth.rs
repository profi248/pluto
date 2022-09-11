use std::time::Duration;
use pluto_network::{ client::Client, prelude::*, topics::* };
use pluto_network::key::{ Keys, Mnemonic, Seed };
use pluto_network::rumqttc::QoS;
use pluto_network::protos::shared::error_response::ErrorType;
use pluto_network::protos::auth::{
    *,
    auth_coordinator_success::*,
};

use crate::NodeError;
use crate::db::Database;

pub fn get_saved_keys() -> Option<Keys> {
    let seed = Database::new().get_by_key("node_seed").ok()??;

    Some(Keys::from_entropy(seed.try_into().ok()?))
}

pub fn restore_keys_from_passphrase(passphrase: String) -> std::result::Result<Keys, pluto_network::key::mnemonic::Error> {
    let mnemonic = Mnemonic::from_passphrase(passphrase)?;
    let seed = Seed::from_mnemonic(mnemonic)?;

    Ok(Keys::from_seed(seed))
}

pub fn save_credentials_to_storage(keys: &Keys) -> Option<()> {
    let seed = keys.seed().entropy().to_vec();
    let db = Database::new();

    db.begin_transaction().ok()?;
    db.set_by_key("node_seed", seed).ok()?;
    db.set_initial_setup_done(true)?;
    db.commit_transaction().ok()?;

    debug!("Node seed saved.");
    Some(())
}

pub fn get_mqtt_client_id() -> String {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("getrandom failed");

    base64::encode(buf)
}

pub async fn register_node(client: &Client, keys: &Keys) -> std::result::Result<(), NodeError> {
    let mut msg = AuthNodeInit::default();
    let pubkey_bytes = keys.public_key().as_bytes().to_vec();
    msg.pubkey = pubkey_bytes.clone();

    let msg = msg.encrypt(&crate::node::COORDINATOR_PUBKEY);

    debug!("Sending registration message to coordinator...");
    let response = client.send_and_listen(
        topic!(Coordinator::Auth).topic(),
        msg,
        QoS::AtMostOnce,
        false,
        topic!(Node::Auth).topic(pluto_network::utils::get_node_topic_id(pubkey_bytes)),
        true,
        Duration::from_secs(3)
    ).await;

    match response {
        Err(e) => {
            Err(NodeError::RequestError(e))
        },
        Ok(response) => {
            let response_enc = response.encrypted()
                .map_err(|_| NodeError::ResponseError(ErrorType::BAD_REQUEST))?;

            let (response, pubkey) = response_enc.decrypt_authenticated(keys.private_key())
                .ok_or(NodeError::ResponseError(ErrorType::CRYPTO_ERROR))?;

            if pubkey != *crate::node::COORDINATOR_PUBKEY {
                return Err(NodeError::ResponseError(ErrorType::CRYPTO_ERROR))
            }

            if let Some(status) = response.auth_status {
                match status {
                    Auth_status::Success(_) => save_credentials_to_storage(keys).ok_or(NodeError::ClientError),
                    Auth_status::Error(e) => Err(
                        NodeError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
                    ),
                    _ => Err(NodeError::ResponseError(ErrorType::BAD_ERROR))
                }
            } else {
                Err(NodeError::ResponseError(ErrorType::BAD_ERROR))
            }
        }
    }
}
