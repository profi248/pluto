use diesel::prelude::*;
use crate::db::{ Database };
use crate::db::schema::backup_job;

#[derive(Queryable, Debug, Identifiable, Default)]
#[primary_key(job_id)]
#[table_name = "backup_job"]
pub struct BackupJob {
    pub job_id: i32,
    pub name: String,
    pub created: i64,
    pub last_ran: Option<i64>,
}
