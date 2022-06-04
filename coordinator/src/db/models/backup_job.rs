use chrono::DateTime;
use chrono::offset::Utc;
use diesel::prelude::*;

use crate::db::schema::backup_job;

#[derive(Queryable, Debug, Identifiable)]
#[primary_key(backup_job_id)]
#[table_name(backup_job)]
pub struct BackupJob {
    pub backup_job_id: i64,
    pub node_id: i64,
    pub created: DateTime<Utc>,
    pub last_ran: Option<DateTime<Utc>>,
    pub total_size: Option<i64>,
    pub name: String,
}
