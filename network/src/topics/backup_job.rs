use crate::protos::backup_job::*;
use super::Request;

impl Request for BackupJobNodePut {
    type Response = BackupJobCoordinatorPutResponse;
}

impl Request for BackupJobNodeListRequest {
    type Response = BackupJobCoordinatorListResponse;
}
