pub mod status;
pub mod setup;
pub mod backup_job;
pub mod node;

use warp::{ reply, http::StatusCode, reply::Json };
use serde_json::json;

pub fn generate_error(error: impl Into<String>, status: StatusCode) -> reply::WithStatus<Json> {
    reply::with_status(reply::json(&json!({ "error": error.into() })), status)
}
