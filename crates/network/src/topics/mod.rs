mod auth;
mod backup_job;

use protobuf::Message as MessageTrait;

use pluto_macros::define_topics;

use crate::protos::auth::*;
use crate::protos::backup_job::*;

define_topics! {
    Coordinator {
        Auth -> "coordinator/auth" => AuthNodeInit,
        ListBackupJobs -> "coordinator/list_jobs" => BackupJobNodeListRequest,
        PutBackupJob -> "coordinator/pub_job" => BackupJobItem
    },
    Node {
        Auth -> "node/{id}/auth",
        ListBackupJobs -> "node/{id}/list_jobs",
        PutBackupJob -> "node/{id}/put_job"
    }
}

/// Defines that a message type is a request, and expects
/// a given message type as a response.
pub trait Request: MessageTrait {
    /// The response message type expected by this request message.
    type Response: MessageTrait;

    /// Simple function to construct a default response object.
    fn response() -> Self::Response {
        Default::default()
    }
}
