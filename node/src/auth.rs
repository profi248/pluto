use std::time::Duration;
use pluto_network::{ prelude::*, topics::*, client::Client };
use pluto_network::key::Keys;
use pluto_network::rumqttc::QoS;
use pluto_network::protos::shared::{ ErrorResponse, error_response::ErrorType };
use pluto_network::protos::auth::{
    *,
    auth_coordinator_success::*,
};


#[derive(Debug)]
pub enum AuthError {
    RequestError,
    ResponseError(ErrorType)
}

pub async fn register_node(client: &Client, keys: &Keys) -> std::result::Result<(), AuthError> {
    let mut msg = AuthNodeInit::default();
    let pubkey_bytes = keys.public_key().as_bytes().to_vec();
    msg.pubkey = pubkey_bytes.clone();

    let msg = msg.encrypt(&crate::node::COORDINATOR_PUBKEY);

    let response = client.send_and_listen(
        topic!(Coordinator::Auth).topic(),
        msg,
        QoS::AtMostOnce,
        false,
        topic!(Node::Auth).topic(pluto_network::utils::get_node_topic_id(pubkey_bytes)),
        true,
        Duration::from_secs(3)
    ).await;

    return match response {
        Err(e) => {
            error!("{e:?}");
            Err(AuthError::RequestError)
        },
        Ok(response) => {
            let response_enc = response.encrypted().map_err(|_| AuthError::ResponseError(ErrorType::BAD_REQUEST))?;
            let response = response_enc.decrypt_authenticated(keys.private_key(), &crate::node::COORDINATOR_PUBKEY)
                .ok_or(AuthError::ResponseError(ErrorType::CRYPTO_ERROR))?;

            if let Some(status) = response.auth_status {
                match status {
                    Auth_status::Success(_) => Ok(()),
                    Auth_status::Error(e) => Err(
                        AuthError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
                    ),
                    _ => todo!()
                }
            } else {
                Err(AuthError::ResponseError(ErrorType::BAD_ERROR))
            }
        }
    }
}
