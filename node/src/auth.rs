use std::time::Duration;
use pluto_network::{ prelude::*, topics::*, client::Client };
use pluto_network::key::{ Keys, Mnemonic, Seed };
use pluto_network::rumqttc::QoS;
use pluto_network::protos::shared::error_response::ErrorType;
use pluto_network::protos::auth::{
    *,
    auth_coordinator_success::*,
};
use crate::auth::AuthError::ClientError;
use crate::db::Database;


#[derive(Debug)]
pub enum AuthError {
    RequestError(Error),
    ResponseError(ErrorType),
    ClientError
}

pub fn get_saved_keys() -> Option<Keys> {
    let seed = Database::get_by_key("node_seed").ok()??;

    Some(Keys::from_entropy(seed.try_into().ok()?))
}

pub fn restore_keys_from_passphrase(passphrase: String) -> std::result::Result<Keys, pluto_network::key::mnemonic::Error> {
    let mnemonic = Mnemonic::from_passphrase(passphrase)?;
    let seed = Seed::from_mnemonic(mnemonic)?;

    Ok(Keys::from_seed(seed))
}

pub fn save_credentials_to_storage(keys: &Keys) -> Option<()> {
    let seed = keys.seed().entropy().to_vec();
    if Database::set_by_key("node_seed",seed).is_ok()
    {
        debug!("Node seed saved.");
        Database::set_initial_setup_done(true)
    } else {
        None
    }
}

pub fn get_mqtt_client_id() -> String {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).expect("getrandom failed");

    base64::encode(buf)
}

pub async fn register_node(client: &Client, keys: &Keys) -> std::result::Result<(), AuthError> {
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
            Err(AuthError::RequestError(e))
        },
        Ok(response) => {
            let response_enc = response.encrypted()
                .map_err(|_| AuthError::ResponseError(ErrorType::BAD_REQUEST))?;

            let response = response_enc.decrypt_authenticated(keys.private_key(),
                                              &crate::node::COORDINATOR_PUBKEY)
                .ok_or(AuthError::ResponseError(ErrorType::CRYPTO_ERROR))?;

            if let Some(status) = response.auth_status {
                match status {
                    Auth_status::Success(_) => save_credentials_to_storage(keys).ok_or(ClientError),
                    Auth_status::Error(e) => Err(
                        AuthError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
                    ),
                    _ => Err(AuthError::ResponseError(ErrorType::BAD_ERROR))
                }
            } else {
                Err(AuthError::ResponseError(ErrorType::BAD_ERROR))
            }
        }
    }
}
