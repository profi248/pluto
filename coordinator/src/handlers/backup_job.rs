use pluto_network::{ prelude::*, topics::*, client::Client, utils::* };

use pluto_network::protos::shared::{ ErrorResponse, error_response::ErrorType };
use pluto_network::protos::backup_job::*;
use pluto_network::protos::backup_job::backup_job_coordinator_list_response::*;
use pluto_network::rumqttc::QoS;

use crate::{ DATABASE, COORDINATOR_PRIVKEY, COORDINATOR_PUBKEY, PublicKey };

pub struct BackupJobListHandler;

#[async_trait::async_trait]
impl Handler for BackupJobListHandler {
    fn topic(&self) -> Topic {
        topic!(Coordinator::ListBackupJobs).into()
    }

    async fn handle(&self, message: Message, client: Client) -> Option<()> {
        let (_, node_pubkey): (BackupJobNodeListRequest, PublicKey) = match message.encrypted() {
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

        let node_topic_id = get_node_topic_id(node_pubkey.as_bytes().to_vec());
        let db = DATABASE.get().unwrap();

        let mut response_msg_wrapper = BackupJobNodeListRequest::response();
        let node_id = match db.get_node_from_pubkey(node_pubkey.as_bytes().to_vec()).await {
            Ok(node) => {
                match node {
                    Some(node) => node.node_id,
                    None => {
                        Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper,
                                         ErrorType::PUBKEY_NOT_FOUND).await;

                        return None;
                    }
                }
            },
            Err(e) => {
                error!("DB failure when getting node from pubkey");
                Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper,
                                 ErrorType::SERVER_ERROR).await;

                return None;
            }
        };

        let backup_jobs = match db.get_backup_jobs_by_node(node_id).await {
            Err(_) => {
                error!("DB failure when getting backup jobs");
                Self::send_error(client, &node_pubkey, node_topic_id, &mut response_msg_wrapper,
                                 ErrorType::SERVER_ERROR).await;

                return None;
            },
            Ok(jobs) => jobs
        };

        debug!("{:?}", backup_jobs);

        let mut list_msg = backup_job_coordinator_list_response::List::default();
        let mut backup_jobs_msgs: Vec<BackupJobItem> = vec![];

        for job in backup_jobs {
            let mut msg = BackupJobItem::default();
            msg.job_id = job.backup_job_id as u32;
            msg.name = job.name;
            msg.created = job.created.timestamp() as u64;
            if let Some(last_ran) = job.last_ran {
                msg.last_ran = last_ran.timestamp() as u64;
            }

            backup_jobs_msgs.push(msg);
        }

        list_msg.backup_jobs = backup_jobs_msgs;
        response_msg_wrapper.list_status = Some(List_status::BackupJobs(list_msg));

        let response_msg_wrapper_enc =
            response_msg_wrapper.encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

        client.send(
            topic!(Node::ListBackupJobs).topic(node_topic_id),
            response_msg_wrapper_enc,
            QoS::AtMostOnce,
            false
        ).await.err().map(|e| debug!("{e:?}"));

        Some(())
    }
}

impl BackupJobListHandler {
    async fn send_error(client: Client, node_pubkey: &PublicKey,
                        node_topic_id: String, response_msg_wrapper: &mut BackupJobCoordinatorListResponse,
                        error: ErrorType) {
        let mut error_msg = ErrorResponse::default();
        error_msg.error = error.into();
        response_msg_wrapper.list_status = Some(List_status::Error(error_msg));

        let response_msg_wrapper_enc =
            response_msg_wrapper.clone().encrypt_authenticated(&node_pubkey, &COORDINATOR_PRIVKEY, &COORDINATOR_PUBKEY);

        client.send(
            topic!(Node::ListBackupJobs).topic(node_topic_id),
            response_msg_wrapper_enc,
            QoS::AtMostOnce,
            false
        ).await.err().map(|e| debug!("{e:?}"));
    }
}
