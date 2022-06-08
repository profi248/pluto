use crate::protos::auth::*;
use super::Request;

impl Request for AuthNodeInit {
    type Response = AuthCoordinatorChallenge;
}

impl Request for AuthCoordinatorChallenge {
    type Response = AuthNodeChallengeResponse;
}
