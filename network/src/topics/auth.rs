use rumqttc::AsyncClient;

use crate::protos::auth::{ AuthNodeInit, AuthCoordinatorChallenge };
use super::Request;

impl Request for AuthNodeInit {
    type Response = AuthCoordinatorChallenge;
}
