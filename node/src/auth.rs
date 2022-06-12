use std::time::Duration;
use pluto_network::{ prelude::*, topics::*, client::Client };
use pluto_network::node::key::Keys;
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
    msg.pubkey = keys.public_key().as_bytes().to_vec();

    let response = client.send_and_listen(
        topic!(Coordinator::Auth).topic(),
        msg,
        QoS::AtMostOnce,
        false,
        Duration::from_secs(3)
    ).await;

    return match response {
        Err(_) => {
            Err(AuthError::RequestError)
        },
        Ok(response) => {
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
