use std::collections::BTreeMap;

use warp::{ reply, http::StatusCode, reply::Json };
use serde_json::json;
use serde::{ Serialize, Deserialize };

use pluto_network::client::Client;
use pluto_network::key::{ self, Keys, Mnemonic, Seed };
use pluto_macros::reject;
use pluto_node::backup_job::*;
use pluto_node::db::Database;
use pluto_node::db::models::{
    backup_job::BackupJob,
    backup_job_path::BackupJobPath,
    backup_job_path::PathType
};

use super::generate_error;
use crate::KeysShared;

#[derive(Serialize, Deserialize, Clone)]
pub struct BackupJobPathItem {
    path_id: Option<i32>,
    path: String,
    path_type: PathType
}

impl<'a> From<BackupJobPath> for BackupJobPathItem {
    fn from(path: BackupJobPath) -> Self {
        BackupJobPathItem {
            path_id: Some(path.path_id),
            path: path.path,
            path_type: match path.path_type {
                0 => PathType::Folder,
                1 => PathType::IgnorePattern,
                _ => unreachable!("Invalid path type enum")
            }
        }
    }
}

#[derive(Deserialize)]
pub struct BackupJobCreate {
    name: String
}

#[derive(Deserialize)]
pub struct BackupJobUpdate {
    name: String,
    last_ran: Option<i64>
}

#[derive(Serialize, Clone)]
pub struct BackupJobItem {
    job: BackupJob,
    paths: Vec<BackupJobPathItem>
}

#[derive(Serialize)]
struct Jobs {
    jobs: Vec<BackupJobItem>
}

#[reject]
pub async fn get_jobs() -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let jobs = Database::new().get_backup_jobs()
        .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    let mut jobs_mapping: BTreeMap<i32, (&BackupJob, Vec<BackupJobPathItem>)>
        = BTreeMap::new();

    // i hope cloning is not gonna be too expensive here
    for (job, path) in jobs.iter() {
        if jobs_mapping.contains_key(&job.job_id) {
            jobs_mapping.get_mut(&job.job_id).unwrap().1.push(path.clone().into());
        } else {
            jobs_mapping.insert(job.job_id, (job, vec![path.clone().into()]));
        }
    }

    let jobs_json: Vec<BackupJobItem> = jobs_mapping.iter().map(|(_, (job, paths))| {
        BackupJobItem {
            job: (*job).clone(),
            paths: (*paths).clone()
        }
    }).collect();

    Ok(reply::with_status(reply::json(&Jobs { jobs: jobs_json }), StatusCode::OK))
}

#[reject]
pub async fn create_job(job: BackupJobCreate, client: Client, keys: KeysShared) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let keys_guard = keys.read().await;
    let keys = keys_guard.as_ref().ok_or(
        generate_error(format!("Setup required"), StatusCode::BAD_REQUEST))?;

    let job_id = create_backup_job(&client, keys, job.name).await
        .map_err(|e| generate_error(format!("Error creating backup job: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true, "job_id": job_id })), StatusCode::OK))
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

#[reject]
pub async fn create_job_path(job_id: i32, path: BackupJobPathItem) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let db = Database::new();
    if db.get_backup_job(job_id)
        .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?
        .is_none()
    {
        return Err(generate_error(format!("Path or backup job not found"), StatusCode::NOT_FOUND));
    }

    db.create_backup_job_path(job_id, path.path, path.path_type)
        .map_err(|e| generate_error(format!("Error creating backup job path: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}

#[reject]
pub async fn update_job_path(job_id: i32, path_id: i32, path: BackupJobPathItem) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let db = Database::new();
    validate_job_and_path(job_id, path_id, &db)?;

    if path_id != path.path_id.unwrap_or(0) {
        return Err(generate_error(format!("Path ID mismatch"), StatusCode::BAD_REQUEST));
    }

    Database::new().update_backup_job_path(path_id, path.path, path.path_type)
        .map_err(|e| generate_error(format!("Error updating backup job path: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}

#[reject]
pub async fn delete_job_path(job_id: i32, path_id: i32) -> Result<impl warp::Reply, reply::WithStatus<Json>> {
    let db = Database::new();
    validate_job_and_path(job_id, path_id, &db)?;

    Database::new().delete_backup_job_path(path_id)
        .map_err(|e| generate_error(format!("Error deleting backup job path: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(reply::with_status(reply::json(&json!({ "success": true })), StatusCode::OK))
}

fn validate_job_and_path(job_id: i32, path_id: i32, db: &Database) -> Result<(), reply::WithStatus<Json>> {
    if !db.backup_job_has_path_id(job_id, path_id)
        .map_err(|e| generate_error(format!("Error: {e:?}"), StatusCode::INTERNAL_SERVER_ERROR))?
    {
        return Err(generate_error(format!("Path or backup job not found"), StatusCode::NOT_FOUND));
    }

    Ok(())
}
