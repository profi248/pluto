use crate::protos::auth::*;
use super::Request;

impl Request for AuthNodeInit {
    type Response = AuthCoordinatorSuccess;
}
