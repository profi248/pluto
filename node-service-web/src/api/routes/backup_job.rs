use warp::{reply, http::StatusCode, reply::Json };
use serde_json::json;
use serde::{ Serialize, Deserialize };

use pluto_network::client::Client;
use pluto_network::key::{ self, Keys, Mnemonic, Seed };
use pluto_node::db::Database;

use pluto_macros::reject;
use pluto_node::backup_job::*;
use pluto_node::db::models::backup_job::BackupJob;

use super::generate_error;
use crate::KeysShared;

#[derive(Deserialize)]
pub struct BackupJobCreate {
    name: String
}

#[derive(Deserialize)]
pub struct BackupJobUpdate {
    name: String,
    last_ran: Option<i64>
}

#[derive(Serialize)]
struct Jobs {
    jobs: Vec<BackupJob>
}

#[reject]
pub async fn get_jobs(client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let jobs = Database::new().get_backup_jobs()
        .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;
    let jobs_json = Jobs { jobs };

    Ok(reply::with_status(reply::json(&jobs_json), StatusCode::OK))
}

#[reject]
pub async fn create_job(job: BackupJobCreate, client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let keys_guard = keys.read().await;
    let keys = keys_guard.as_ref().ok_or(
        generate_error(format!("Setup required"), StatusCode::BAD_REQUEST))?;

    create_backup_job(&client, keys, job.name).await
        .map_err(|e| generate_error(format!("Error creating backup job: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}

#[reject]
pub async fn update_job(job_id: i32, job: BackupJobUpdate, client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let keys_guard = keys.read().await;
    let keys = keys_guard.as_ref().ok_or(
        generate_error(format!("Setup required"), StatusCode::BAD_REQUEST))?;

    update_backup_job(&client, keys, job_id, job.name, job.last_ran).await
        .map_err(|e| generate_error(format!("Error updating backup job: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}

#[reject]
pub async fn delete_job(job_id: i32, client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let keys_guard = keys.read().await;
    let keys = keys_guard.as_ref().ok_or(
        generate_error(format!("Setup required"), StatusCode::BAD_REQUEST))?;

    delete_backup_job(&client, keys, job_id).await
        .map_err(|e| generate_error(format!("Error deleting backup job: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}
