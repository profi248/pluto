use pluto_network::{ client::Client, key::Keys, rumqttc::QoS, prelude::*, topics::*, utils::get_node_topic_id };
use pluto_network::protos::shared::error_response::ErrorType;
use pluto_network::protos::backup_job::{ *, backup_job_coordinator_list_response::* };

use crate::{ node, NodeError };
use crate::db::models::backup_job::BackupJob;

pub async fn get_remote_backup_jobs(client: &Client, keys: &Keys) -> std::result::Result<Vec<BackupJob>, NodeError> {
    let node_topic_id = get_node_topic_id(keys.public_key().as_bytes().to_vec());

    let response = client.send_and_listen(
        topic!(Coordinator::ListBackupJobs).topic(),
        pluto_network::protos::backup_job::BackupJobNodeListRequest::default()
            .encrypt_authenticated(&node::COORDINATOR_PUBKEY, keys.private_key(), keys.public_key()),
        QoS::AtMostOnce,
        false,
        topic!(Node::ListBackupJobs).topic(node_topic_id),
        true,
        std::time::Duration::from_secs(3)
    ).await?;

    let msg =
        response.encrypted().map_err(|_| NodeError::ResponseError(ErrorType::CRYPTO_ERROR))?;

    let (msg, pubkey) = msg.decrypt_authenticated(keys.private_key())
        .ok_or(NodeError::ResponseError(ErrorType::CRYPTO_ERROR))?;

    if pubkey != *crate::node::COORDINATOR_PUBKEY {
        return Err(NodeError::ResponseError(ErrorType::CRYPTO_ERROR))
    }

    let jobs;
    if let Some(status) = msg.list_status {
        jobs = match status {
            List_status::BackupJobs(jobs) => jobs,
            List_status::Error(e) => return Err(
                NodeError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
            ),
            _ => return Err(NodeError::ResponseError(ErrorType::BAD_ERROR))
        };
    } else {
        return Err(NodeError::ResponseError(ErrorType::BAD_ERROR))
    }

    let mut job_vec: Vec<BackupJob> = vec![];
    for job_msg in jobs.backup_jobs {
        let mut job = BackupJob::default();

        job.job_id = job_msg.job_id as i32;
        job.name = job_msg.name;
        job.created = job_msg.created as i64;
        job.last_run = if job_msg.last_ran != 0 {
            Some(job_msg.last_ran as i64)
        } else {
            None
        };

        job_vec.push(job);
    }

    return Ok(job_vec)
}
