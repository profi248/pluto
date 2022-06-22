use pluto_network::{client::Client, key::Keys, rumqttc::QoS, prelude::*, topics::*, utils::get_node_topic_id };
use pluto_network::protos::shared::error_response::ErrorType;
use pluto_network::protos::backup_job::{
    *,
    backup_job_coordinator_list_response::*,
    backup_job_coordinator_put_response::*
};

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
        response.encrypted().map_err(|_| NodeError::CryptoError)?;

    let (msg, pubkey) = msg.decrypt_authenticated(keys.private_key())
        .ok_or(NodeError::CryptoError)?;

    if pubkey != *crate::node::COORDINATOR_PUBKEY {
        return Err(NodeError::CryptoError)
    }

    let jobs;
    if let Some(status) = msg.list_status {
        jobs = match status {
            List_status::BackupJobs(jobs) => jobs,
            List_status::Error(e) => return Err(
                NodeError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
            ),
            _ => return Err(NodeError::ParseError)
        };
    } else {
        return Err(NodeError::ParseError)
    }

    let mut job_vec: Vec<BackupJob> = vec![];
    for job_msg in jobs.backup_jobs {
        let mut job = BackupJob::default();

        job.job_id = job_msg.job_id as i32;
        job.name = job_msg.name;
        job.created = job_msg.created as i64;
        job.last_ran = if job_msg.last_ran != 0 {
            Some(job_msg.last_ran as i64)
        } else {
            None
        };

        job_vec.push(job);
    }

    return Ok(job_vec)
}

pub async fn send_backup_job_to_coordinator(client: &Client, keys: &Keys, job: BackupJob) -> std::result::Result<(), NodeError> {
    let node_topic_id = get_node_topic_id(keys.public_key().as_bytes().to_vec());

    let mut msg = pluto_network::protos::backup_job::BackupJobItem::default();
    msg.job_id = job.job_id as u32;
    msg.name = job.name;
    msg.created = job.created as u64;
    msg.last_ran = job.last_ran.unwrap_or(0) as u64;

    let response = client.send_and_listen(
        topic!(Coordinator::PutBackupJob).topic(),
        msg.encrypt_authenticated(&node::COORDINATOR_PUBKEY, keys.private_key(), keys.public_key()),
        QoS::AtMostOnce,
        false,
        topic!(Node::PutBackupJob).topic(node_topic_id),
        true,
        std::time::Duration::from_secs(3)
    ).await?;

    let msg =
        response.encrypted().map_err(|_| NodeError::CryptoError)?;

    let (msg, pubkey) = msg.decrypt_authenticated(keys.private_key())
        .ok_or(NodeError::CryptoError)?;

    if pubkey != *crate::node::COORDINATOR_PUBKEY {
        return Err(NodeError::CryptoError)
    }

    match msg.put_status {
        Some(Put_status::Success(succ)) => {
            if succ.success {
                Ok(())
            } else {
                Err(NodeError::ParseError)
            }
        },
        Some(Put_status::Error(e)) => Err(
            NodeError::ResponseError(e.error.enum_value_or(ErrorType::BAD_ERROR))
        ),
        Some(_) => Err(NodeError::ParseError),
        None => Err(NodeError::ParseError)
    }
}
