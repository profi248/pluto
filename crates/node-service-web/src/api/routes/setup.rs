use warp::{ reply, http::StatusCode, reply::Json };
use serde::{ Deserialize };
use serde_json::json;

use pluto_network::client::Client;
use pluto_network::key::{ self, Keys, Mnemonic, Seed };
use pluto_node::db::Database;
use pluto_node::utils::subscribe_to_topics;

use pluto_macros::reject;

use super::generate_error;
use crate::KeysShared;

#[derive(Deserialize)]
pub struct Setup {
    mnemonic: Option<String>
}

#[reject]
pub async fn setup(setup: Setup, client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    match Database::new().get_initial_setup_done() {
        Some(false) => {},
        Some(true) => {
            return Err(generate_error("Setup already completed", StatusCode::BAD_REQUEST));
        },
        None => {
            return Err(generate_error("Database error", StatusCode::INTERNAL_SERVER_ERROR));
        }
    }

    if keys.read().await.is_some() {
        return Err(generate_error("Inconsistent keys state", StatusCode::INTERNAL_SERVER_ERROR));
    }

    let passphrase = if let Some(passphrase) = setup.mnemonic {
        let mnemonic = Mnemonic::from_passphrase(passphrase.clone()).map_err(|e| match e {
            key::mnemonic::Error::InvalidWord(w) => generate_error(format!("Invalid passphrase word: {w}"), StatusCode::BAD_REQUEST),
            key::mnemonic::Error::InvalidLength => generate_error("Invalid passphrase length", StatusCode::BAD_REQUEST),
            _ => generate_error("Server error", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let seed = Seed::from_mnemonic(mnemonic).map_err(|e| match e {
            key::mnemonic::Error::InvalidChecksum => generate_error("Invalid passphrase checksum", StatusCode::BAD_REQUEST),
            _ => generate_error("Server error", StatusCode::INTERNAL_SERVER_ERROR)
        })?;

        let new_keys = Keys::from_seed(seed);
        *keys.write().await = Some(new_keys.clone());

        subscribe_to_topics(client.clone(), &new_keys).await
            .map_err(|e| generate_error(format!("Error when subscribing to topics: {e:?}"),
                StatusCode::INTERNAL_SERVER_ERROR))?;
        match pluto_node::auth::save_credentials_to_storage(&new_keys) {
            Some(_) => {},
            None => {
                return Err(generate_error("Error when saving keys to storage", StatusCode::INTERNAL_SERVER_ERROR));
            }
        };

        None
    } else {
        let new_keys = Keys::generate();
        *keys.write().await = Some(new_keys.clone());
        subscribe_to_topics(client.clone(), &new_keys).await
            .map_err(|e| generate_error(format!("Error when subscribing to topics: {e:?}"),
                StatusCode::INTERNAL_SERVER_ERROR))?;

        pluto_node::auth::register_node(&client, &new_keys).await
            .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

        Some(new_keys.seed().to_mnemonic().to_passphrase())
    };

    Ok(reply::with_status(reply::json(&json!({
        "success": true,
        "passphrase": passphrase
    })), StatusCode::OK))
}
