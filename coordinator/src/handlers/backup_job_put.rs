use chrono::{DateTime, NaiveDateTime, Utc};
use pluto_network::{prelude::*, topics::*, client::Client, utils::* };

use pluto_network::protos::shared::{ ErrorResponse, error_response::ErrorType };
use pluto_network::protos::backup_job::*;
use pluto_network::protos::backup_job::backup_job_coordinator_put_response::*;
use pluto_network::rumqttc::QoS;

use crate::{DATABASE, COORDINATOR_PRIVKEY, COORDINATOR_PUBKEY, PublicKey, Coordinator};
use crate::db::models::BackupJob;
use crate::logic::backup_job::BackupJobError;

pub struct BackupJobPutHandler;

#[async_trait::async_trait]
impl Handler for BackupJobPutHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::PutBackupJob).into()
    }

    async fn handle(&self, message: Message, client: Client) -> Option<()> {
        let (job_msg, node_pubkey): (BackupJobItem, PublicKey) = match message.encrypted() {
            Ok(m) => {
                match m.decrypt_authenticated(&COORDINATOR_PRIVKEY) {
                    Some(m) => m,
                    // todo return a error message
                    None => {
                        warn!("failed to decrypt");
                        return None;
                    }
                }
            },
            // ignore malformed messages for now
            Err(_) => {
                warn!("not encrypted");
                return None;
            },
        };

        let db = DATABASE.get().unwrap();

        let mut job: BackupJob = BackupJob::default();
        let node_topic_id = get_node_topic_id(node_pubkey.as_bytes().to_vec());
        let mut response_msg_wrapper = BackupJobItem::response();

        job.node_id = match db.get_node_from_pubkey(node_pubkey.as_bytes().to_vec()).await {
            Ok(Some(node)) => node.node_id,
            Ok(None) => {
                Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper,
                                 ErrorType::PUBKEY_NOT_FOUND).await;
                return None;
            },
            Err(_) => {
                error!("DB failure when finding node");

                Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper,
                                 ErrorType::SERVER_ERROR).await;
                return None;
            }
        };

        job.local_job_id = job_msg.job_id as i32;
        job.created = DateTime::from_utc(NaiveDateTime::from_timestamp(job_msg.created as i64, 0), Utc);

        job.last_ran = if job_msg.last_ran != 0 {
             Some(DateTime::from_utc(NaiveDateTime::from_timestamp(job_msg.last_ran as i64, 0), Utc))
        } else {
            None
        };

        job.total_size = None; // todo
        job.name = job_msg.name;

        match Coordinator::insert_or_update_backup_job(db, job).await {
            Ok(_) => {
                let mut success_msg = Success::default();
                success_msg.success = true;
                response_msg_wrapper.put_status = Some(Put_status::Success(success_msg));

                let response_msg_wrapper_enc =
                    response_msg_wrapper.encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

                client.send(
                    topic!(Node::PutBackupJob).topic(node_topic_id),
                    response_msg_wrapper_enc,
                    QoS::AtMostOnce,
                    false
                ).await.err().map(|e| debug!("{e:?}"));
            },
            Err(e) => {
                let err = match e {
                    BackupJobError::DatabaseError => ErrorType::SERVER_ERROR,
                    BackupJobError::NameTooLong => ErrorType::INVALID_BACKUP_JOB
                };

                Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper, err).await;
                return None;
            }
        };

        Some(())
    }
}

impl BackupJobPutHandler {
    async fn send_error(client: Client, node_pubkey: &PublicKey,
                        node_topic_id: String, response_msg_wrapper: &mut BackupJobCoordinatorPutResponse,
                        error: ErrorType) {
        let mut error_msg = ErrorResponse::default();
        error_msg.error = error.into();
        response_msg_wrapper.put_status = Some(Put_status::Error(error_msg));

        let response_msg_wrapper_enc =
            response_msg_wrapper.clone().encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

        client.send(
            topic!(Node::PutBackupJob).topic(node_topic_id),
            response_msg_wrapper_enc,
            QoS::AtMostOnce,
            false
        ).await.err().map(|e| debug!("{e:?}"));
    }
}
